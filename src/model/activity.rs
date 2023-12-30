use serde::{Serialize, Deserialize};

use super::object::Object;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
	#[serde(rename = "@context")]
	context: String,

	id: String,

	#[serde(rename = "type")]
	activity_type: ActivityType,

	actor: String,

	object: Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
	Create,
}

#[cfg(test)]
mod test {
	use super::{Activity, ActivityType};
	use crate::model::object::Object;

	#[test]
	fn activity_serializes_as_expected() {
		let activity = Activity {
			context: "https://www.w3.org/ns/activitystreams".into(),
			id: "https://my-example.com/create-hello-world".into(),
			activity_type: ActivityType::Create,
			actor: "https://my-example.com/actor".into(),
			object: Object::default(),
		};

		let serialized_activity = serde_json::to_string(&activity).unwrap();
		let expected_serialized_activity = "{\"@context\":\"https://www.w3.org/ns/activitystreams\",\"id\":\"https://my-example.com/create-hello-world\",\"type\":\"Create\",\"actor\":\"https://my-example.com/actor\",\"object\":{\"id\":\"https://my-example.com/hello-world\",\"type\":\"Note\",\"published\":\"2018-06-23T17:17:11Z\",\"attributedTo\":\"https://my-example.com/actor\",\"inReplyTo\":\"https://mastodon.social/@Gargron/100254678717223630\",\"content\":\"<p>Hello world</p>\",\"to\":\"https://www.w3.org/ns/activitystreams#Public\"}}";

		assert_eq!(serialized_activity, expected_serialized_activity);
	}
}
