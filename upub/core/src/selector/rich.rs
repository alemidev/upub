use apb::{ActivityMut, LinkMut, ObjectMut};
use sea_orm::{DbErr, EntityName, FromQueryResult, Iden, QueryResult};


pub struct RichActivity {
	pub activity: crate::model::activity::Model,
	pub object: Option<crate::model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<crate::model::attachment::Model>>,
	pub hashtags: Option<Vec<crate::model::hashtag::Model>>,
	pub mentions: Option<Vec<crate::model::mention::Model>>,
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
		let object = match self.object {
			None => apb::Node::maybe_link(self.activity.object.clone()),
			Some(o) => {
				// TODO can we avoid repeating this tags code?
				let mut tags = Vec::new();
				if let Some(mentions) = self.mentions {
					for mention in mentions {
						tags.push(
							apb::new()
								.set_link_type(Some(apb::LinkType::Mention))
								.set_href(&mention.actor)
								// TODO do i need to set name? i could join while batch loading or put the @name in
								// each mention object...
						);
					}
				}
				if let Some(hashtags) = self.hashtags {
					for hash in hashtags {
						tags.push(
							// TODO ewwww set_name clash and cant use builder, wtf is this
							LinkMut::set_name(apb::new(), Some(&format!("#{}", hash.name)))
								.set_link_type(Some(apb::LinkType::Hashtag))
								// TODO do we need to set href too? we can't access context here, quite an issue!
						);
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
	pub hashtags: Option<Vec<crate::model::hashtag::Model>>,
	pub mentions: Option<Vec<crate::model::mention::Model>>,
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
		// TODO can we avoid repeating this tags code?
		let mut tags = Vec::new();
		if let Some(mentions) = self.mentions {
			for mention in mentions {
				tags.push(
					apb::new()
						.set_link_type(Some(apb::LinkType::Mention))
						.set_href(&mention.actor)
						// TODO do i need to set name? i could join while batch loading or put the @name in
						// each mention object...
				);
			}
		}
		if let Some(hashtags) = self.hashtags {
			for hash in hashtags {
				tags.push(
					// TODO ewwww set_name clash and cant use builder, wtf is this
					LinkMut::set_name(apb::new(), Some(&format!("#{}", hash.name)))
						.set_link_type(Some(apb::LinkType::Hashtag))
						// TODO do we need to set href too? we can't access context here, quite an issue!
				);
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
