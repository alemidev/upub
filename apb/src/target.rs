use crate::Object;

pub const PUBLIC : &str = "https://www.w3.org/ns/activitystreams#Public";

pub trait Addressed {
	fn addressed(&self) -> Vec<String>; // TODO rename this? remate others? idk
	// fn primary_targets(&self) -> Vec<String>;
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

	// fn primary_targets(&self) -> Vec<String> {
	// 	let mut to : Vec<String> = self.to().ids();
	// 	to.append(&mut self.bto().ids());
	// 	to
	// }

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
