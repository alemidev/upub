use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webfinger {
	subject: String,
	links: Vec<WebfingerLink>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebfingerLink {
	rel: String,
	#[serde(rename = "type")]
	link_type: String,
	href: String,
}

#[cfg(test)]
mod test {
	use super::{Webfinger, WebfingerLink};

	#[test]
	fn webfinger_serializes_as_expected() {
		let webfinger = Webfinger {
			subject: "acct:alice@my-example.com".into(),
			links: vec![
				WebfingerLink {
					rel: "self".into(),
					link_type: "application/activity+json".into(),
					href: "https://my-example.com/actor".into(),
				},
			],
		};

		let serialized_webfinger = serde_json::to_string(&webfinger).unwrap();
		let expected_serialized_webfinger = "{\"subject\":\"acct:alice@my-example.com\",\"links\":[{\"rel\":\"self\",\"type\":\"application/activity+json\",\"href\":\"https://my-example.com/actor\"}]}";

		assert_eq!(serialized_webfinger, expected_serialized_webfinger);
	}
}
