use apb::ActivityMut;
use sea_orm::{DbErr, EntityName, FromQueryResult, Iden, QueryResult};

use crate::ext::IntoActivityPub;

pub struct RichMention {
	pub mention: crate::model::mention::Model,
	pub id: String,
	pub fqn: String,
}

impl IntoActivityPub for RichMention {
	fn into_activity_pub_json(self, _ctx: &crate::Context) -> serde_json::Value {
		use apb::LinkMut;
		apb::new()
			.set_link_type(Some(apb::LinkType::Mention))
			.set_href(Some(self.id))
			.set_name(Some(self.fqn))
	}
}

pub struct RichHashtag {
	pub hash: crate::model::hashtag::Model,
}

impl IntoActivityPub for RichHashtag {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		use apb::LinkMut;
		apb::new()
			.set_name(Some(format!("#{}", self.hash.name)))
			.set_link_type(Some(apb::LinkType::Hashtag))
			.set_href(Some(crate::url!(ctx, "/tags/{}", self.hash.name)))
	}
}

pub struct RichObject {
	pub object: Option<crate::model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<crate::model::attachment::Model>>,
	pub hashtags: Option<Vec<RichHashtag>>,
	pub mentions: Option<Vec<RichMention>>,
}

impl FromQueryResult for RichObject {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichObject {
			attachments: None,
			hashtags: None,
			mentions: None,
			liked: res.try_get(crate::model::like::Entity.table_name(), &crate::model::like::Column::Actor.to_string()).ok(),
			object: crate::model::object::Model::from_query_result_optional(res, crate::model::object::Entity.table_name())?,
		})
	}
}

impl IntoActivityPub for RichObject {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		use apb::ObjectMut;
		match self.object {
			Some(object) => {
				let mut tags = Vec::new();
				if let Some(mentions) = self.mentions {
					for mention in mentions {
						tags.push(mention.into_activity_pub_json(ctx));
					}
				}
				if let Some(hashtags) = self.hashtags {
					for hash in hashtags {
						tags.push(hash.into_activity_pub_json(ctx));
					}
				}
				object.into_activity_pub_json(ctx)
					.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
					.set_tag(apb::Node::maybe_array(tags))
					.set_attachment(match self.attachments {
						None => apb::Node::Empty,
						Some(vec) => apb::Node::array(
							vec.into_iter()
								.map(|x| x.into_activity_pub_json(ctx))
								.collect()
						),
					})
			},
			None => serde_json::Value::Null,
		}
	}
}

pub struct RichActivity {
	pub activity: Option<crate::model::activity::Model>,
	pub object: RichObject,
	pub discovered: chrono::DateTime<chrono::Utc>,
}

impl FromQueryResult for RichActivity {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichActivity {
			object: RichObject::from_query_result(res, _pre)?,
			activity: crate::model::activity::Model::from_query_result_optional(res, crate::model::activity::Entity.table_name())?,
			discovered: res.try_get(
				crate::model::addressing::Entity.table_name(),
				&crate::model::addressing::Column::Published.to_string()
			).unwrap_or(chrono::Utc::now()),
		})
	}
}

impl IntoActivityPub for RichActivity {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		use apb::ObjectMut;
		match (self.activity, &self.object.object) {
			(None, None) => serde_json::Value::Null,

			(Some(activity), None) => activity.into_activity_pub_json(ctx),

			(None, Some(ref _object)) => {
				apb::new()
					.set_activity_type(Some(apb::ActivityType::View))
					.set_published(Some(self.discovered))
					.set_object(apb::Node::object(self.object.into_activity_pub_json(ctx)))
			},

			(Some(activity), Some(ref _object)) => {
				activity
					.into_activity_pub_json(ctx)
					.set_object(apb::Node::object(self.object.into_activity_pub_json(ctx)))
			},
		}
	}
}

pub struct RichNotification {
	pub activity: crate::model::activity::Model,
	pub seen: bool,
}

impl FromQueryResult for RichNotification {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichNotification {
			activity: crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name())?,
			seen: res.try_get(
				crate::model::notification::Entity.table_name(),
				&crate::model::notification::Column::Seen.to_string()
			).unwrap_or(false),
		})
	}
}

impl IntoActivityPub for RichNotification {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		let seen = self.seen;
		self.activity.into_activity_pub_json(ctx)
			.set_seen(Some(seen))
	}
}
