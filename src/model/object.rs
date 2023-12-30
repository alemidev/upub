use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
	id: String,

	#[serde(rename = "type")]
	object_type: ObjectType,

	published: DateTime<Utc>,

	#[serde(rename = "attributedTo")]
	attributed_to: String,

	#[serde(rename = "inReplyTo")]
	in_reply_to: String,

	content: String,

	to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
	Note,
}

#[cfg(test)]
impl Default for Object {
	fn default() -> Self {
		Object {
			id: "https://my-example.com/hello-world".into(),
			object_type: ObjectType::Note,
			published: DateTime::parse_from_rfc3339("2018-06-23T17:17:11Z").unwrap().into(),
			attributed_to: "https://my-example.com/actor".into(),
			in_reply_to: "https://mastodon.social/@Gargron/100254678717223630".into(),
			content: "<p>Hello world</p>".into(),
			to: "https://www.w3.org/ns/activitystreams#Public".into(),
		}
	}
}

#[cfg(test)]
mod test {
	use super::Object;

	#[test]
	fn object_serializes_as_expected() {
		let object = Object::default();

		let serialized_object = serde_json::to_string(&object).unwrap();
		let expected_serialized_object = "{\"id\":\"https://my-example.com/hello-world\",\"type\":\"Note\",\"published\":\"2018-06-23T17:17:11Z\",\"attributedTo\":\"https://my-example.com/actor\",\"inReplyTo\":\"https://mastodon.social/@Gargron/100254678717223630\",\"content\":\"<p>Hello world</p>\",\"to\":\"https://www.w3.org/ns/activitystreams#Public\"}";

		assert_eq!(serialized_object, expected_serialized_object);
	}
}
