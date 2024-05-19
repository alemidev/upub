use crate::{Object, Link};

pub const PUBLIC : &str = "https://www.w3.org/ns/activitystreams#Public";

pub trait Addressed {
	fn addressed(&self) -> Vec<String>;
}

impl<T: Object> Addressed for T {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to().map(|x| x.href().to_string()).collect();
		to.append(&mut self.bto().map(|x| x.href().to_string()).collect());
		to.append(&mut self.cc().map(|x| x.href().to_string()).collect());
		to.append(&mut self.bcc().map(|x| x.href().to_string()).collect());
		to
	}
}
