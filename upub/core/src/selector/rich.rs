use apb::ActivityMut;
use sea_orm::{DbErr, EntityName, FromQueryResult, Iden, QueryResult};

pub struct RichMention {
	pub mention: crate::model::mention::Model,
	pub id: String,
	pub fqn: String,
}

impl RichMention {
	pub fn ap(self) -> serde_json::Value {
		use apb::LinkMut;
		apb::new()
			.set_link_type(Some(apb::LinkType::Mention))
			.set_href(&self.id)
			.set_name(Some(&self.fqn))
	}
}

pub struct RichHashtag {
	pub hash: crate::model::hashtag::Model,
}

impl RichHashtag {
	pub fn ap(self) -> serde_json::Value {
		use apb::LinkMut;
		apb::new()
			.set_name(Some(&format!("#{}", self.hash.name)))
			.set_link_type(Some(apb::LinkType::Hashtag))
	}
}

pub struct RichActivity {
	pub activity: crate::model::activity::Model,
	pub object: Option<crate::model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<crate::model::attachment::Model>>,
	pub hashtags: Option<Vec<RichHashtag>>,
	pub mentions: Option<Vec<RichMention>>,
}

impl FromQueryResult for RichActivity {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichActivity {
			activity: crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name())?,
			object: crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name()).ok(),
			liked: res.try_get(crate::model::like::Entity.table_name(), &crate::model::like::Column::Actor.to_string()).ok(),
			attachments: None, hashtags: None, mentions: None,
		})
	}
}

impl RichActivity {
	pub fn ap(self) -> serde_json::Value {
		use apb::ObjectMut;
		let object = match self.object {
			None => apb::Node::maybe_link(self.activity.object.clone()),
			Some(o) => {
				// TODO can we avoid repeating this tags code?
				let mut tags = Vec::new();
				if let Some(mentions) = self.mentions {
					for mention in mentions {
						tags.push(mention.ap());
					}
				}
				if let Some(hashtags) = self.hashtags {
					for hash in hashtags {
						tags.push(hash.ap());
					}
				}
				apb::Node::object(
					o.ap()
						.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
						.set_tag(apb::Node::array(tags))
						.set_attachment(match self.attachments {
							None => apb::Node::Empty,
							Some(vec) => apb::Node::array(
								vec.into_iter().map(|x| x.ap()).collect()
							),
						})
				)
			},
		};
		self.activity.ap().set_object(object)
	}
}

pub struct RichObject {
	pub object: crate::model::object::Model,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<crate::model::attachment::Model>>,
	pub hashtags: Option<Vec<RichHashtag>>,
	pub mentions: Option<Vec<RichMention>>,
}

impl FromQueryResult for RichObject {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichObject {
			object: crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name())?,
			liked: res.try_get(crate::model::like::Entity.table_name(), &crate::model::like::Column::Actor.to_string()).ok(),
			attachments: None, hashtags: None, mentions: None,
		})
	}
}

impl RichObject {
	pub fn ap(self) -> serde_json::Value {
		use apb::ObjectMut;
		// TODO can we avoid repeating this tags code?
		let mut tags = Vec::new();
		if let Some(mentions) = self.mentions {
			for mention in mentions {
				tags.push(mention.ap());
			}
		}
		if let Some(hashtags) = self.hashtags {
			for hash in hashtags {
				tags.push(hash.ap());
			}
		}
		self.object.ap()
			.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
			.set_tag(apb::Node::array(tags))
			.set_attachment(match self.attachments {
				None => apb::Node::Empty,
				Some(vec) => apb::Node::array(
					vec.into_iter().map(|x| x.ap()).collect()
				)
			})
	}
}
