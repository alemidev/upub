use crate::{Object, Link};

pub const PUBLIC : &str = "https://www.w3.org/ns/activitystreams#Public";

pub trait Addressed {
	fn addressed(&self) -> Vec<String>;
}

impl<T: Object> Addressed for T {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to().ids();
		to.append(&mut self.bto().ids());
		to.append(&mut self.cc().ids());
		to.append(&mut self.bcc().ids());
		to
	}
}
