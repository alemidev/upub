use crate::Object;

pub const PUBLIC : &str = "https://www.w3.org/ns/activitystreams#Public";

pub trait Addressed {
	fn addressed(&self) -> Vec<String>; // TODO rename this? remate others? idk
	fn mentioning(&self) -> Vec<String>;
	// fn secondary_targets(&self) -> Vec<String>;
	// fn public_targets(&self) -> Vec<String>;
	// fn private_targets(&self) -> Vec<String>;
}

impl<T: Object> Addressed for T {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to().all_ids();
		to.append(&mut self.bto().all_ids());
		to.append(&mut self.cc().all_ids());
		to.append(&mut self.bcc().all_ids());
		to
	}

	fn mentioning(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to().all_ids();
		to.append(&mut self.bto().all_ids());
		to
	}

	// fn secondary_targets(&self) -> Vec<String> {
	// 	let mut to : Vec<String> = self.cc().ids();
	// 	to.append(&mut self.bcc().ids());
	// 	to
	// }

	// fn public_targets(&self) -> Vec<String> {
	// 	let mut to : Vec<String> = self.to().ids();
	// 	to.append(&mut self.cc().ids());
	// 	to
	// }

	// fn private_targets(&self) -> Vec<String> {
	// 	let mut to : Vec<String> = self.bto().ids();
	// 	to.append(&mut self.bcc().ids());
	// 	to
	// }
}

#[cfg(test)]
mod test {
	use super::Addressed;

	#[test]
	#[cfg(feature = "unstructured")]
	fn addressed_trait_finds_all_targets_on_json_objects() {
		let obj = serde_json::json!({
			"id": "http://localhost:8080/obj/1",
			"type": "Note",
			"content": "hello world!",
			"published": "2024-06-04T17:09:20+00:00",
			"to": ["http://localhost:8080/usr/root/followers"],
			"bto": ["https://localhost:8080/usr/secret"],
			"cc": [crate::target::PUBLIC],
			"bcc": [],
		});

		let addressed = obj.addressed();

		assert_eq!(
			addressed,
			vec![
				"http://localhost:8080/usr/root/followers".to_string(), 
				"https://localhost:8080/usr/secret".to_string(),
				crate::target::PUBLIC.to_string(),
			]
		);
	}

	#[test]
	#[cfg(feature = "unstructured")]
	fn primary_targets_only_finds_to_and_bto() {
		let obj = serde_json::json!({
			"id": "http://localhost:8080/obj/1",
			"type": "Note",
			"content": "hello world!",
			"published": "2024-06-04T17:09:20+00:00",
			"to": ["http://localhost:8080/usr/root/followers"],
			"bto": ["https://localhost:8080/usr/secret"],
			"cc": [crate::target::PUBLIC],
			"bcc": [],
		});

		let addressed = obj.mentioning();

		assert_eq!(
			addressed,
			vec![
				"http://localhost:8080/usr/root/followers".to_string(), 
				"https://localhost:8080/usr/secret".to_string(),
			]
		);
	}
}
