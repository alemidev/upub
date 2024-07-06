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
			.set_href(Some(&self.id))
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
	pub activity: Option<crate::model::activity::Model>,
	pub object: Option<crate::model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<crate::model::attachment::Model>>,
	pub hashtags: Option<Vec<RichHashtag>>,
	pub mentions: Option<Vec<RichMention>>,
	pub discovered: chrono::DateTime<chrono::Utc>,
}

impl FromQueryResult for RichActivity {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichActivity {
			attachments: None, hashtags: None, mentions: None,
			discovered: res.try_get(crate::model::addressing::Entity.table_name(), &crate::model::addressing::Column::Published.to_string())?,
			liked: res.try_get(crate::model::like::Entity.table_name(), &crate::model::like::Column::Actor.to_string()).ok(),
			object: crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name()).ok(),
			activity: crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name()).ok(),
		})
	}
}

impl RichActivity {
	pub fn ap(self) -> serde_json::Value {
		use apb::ObjectMut;
		match (self.activity, self.object) {
			(None, None) => serde_json::Value::Null,

			(Some(activity), None) => {
				let obj = apb::Node::maybe_link(activity.object.clone());
				activity.ap().set_object(obj)
			},

			(maybe_activity, Some(object)) => {
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

				let activity = match maybe_activity {
					Some(activity) => activity.ap(),
					None => apb::new()
						.set_activity_type(Some(apb::ActivityType::View))
						.set_published(Some(self.discovered))
				};

				activity
					.set_object(apb::Node::object(
						object.ap()
							.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
							.set_tag(apb::Node::maybe_array(tags))
							.set_attachment(match self.attachments {
								None => apb::Node::Empty,
								Some(vec) => apb::Node::array(
									vec.into_iter().map(|x| x.ap()).collect()
								),
							})
					))
			},
		}
	}

	// TODO ughhh cant make it a trait because there's this different one!!!
	pub fn object_ap(self) -> serde_json::Value {
		use apb::ObjectMut;
		match self.object {
			Some(object) => {
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
				object.ap()
					.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
					.set_tag(apb::Node::maybe_array(tags))
					.set_attachment(match self.attachments {
						None => apb::Node::Empty,
						Some(vec) => apb::Node::array(
							vec.into_iter().map(|x| x.ap()).collect()
						),
					})
			},
			None => serde_json::Value::Null,
		}
	}
}
