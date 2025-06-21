pub mod view;
pub mod members;
pub mod feed;
pub mod item;

use leptos::prelude::*;
use apb::{ActivityMut, Collection};

#[derive(Clone, Copy)]
pub struct ListControls {
	pub active: ReadSignal<Option<String>>,
	pub set_active: WriteSignal<Option<String>>,

	pub available: ReadSignal<Vec<String>>,
	pub set_available: WriteSignal<Vec<String>>,
}

impl ListControls {
	pub fn active_name(&self) -> Option<String> {
		use crate::Cache;
		use apb::Object;

		let active = self.active.get()?;
		if let Some(doc) = crate::cache::OBJECTS.get(&active) {
			if let Ok(name) = doc.name() {
				return Some(name);
			}
		}
		Some(active)
	}

	pub fn list_name(&self, id: &str) -> String {
		use crate::Cache;
		use apb::Object;
		if let Some(doc) = crate::cache::OBJECTS.get(id) {
			if let Ok(name) = doc.name() {
				return name;
			}
		}
		id.to_string()
	}

	pub fn fetch(&self, auth: crate::Auth) {
		use apb::CollectionPage;

		let set_available = self.set_available;
		if let Some(uid) = auth.userid.get() {
			leptos::task::spawn_local(async move {
				let mut lists = Vec::new();
				let mut next = format!("{uid}/lists/page");
				loop {
					match crate::Http::fetch::<serde_json::Value>(&next, auth).await {
						Ok(page) => {
							for doc in page.ordered_items().flat() {
								if let Ok(id) = doc.id() {
									lists.push(id);
								}
								if let Ok(obj) = doc.into_inner() {
									crate::cache::OBJECTS.include(std::sync::Arc::new(obj));
								}
							}
							if let Ok(next_page) = page.next().id() {
								next = next_page;
								continue;
							}
						},
						Err(e) => {
							tracing::error!("error fetching user lists: {e}");
						},
					}
					break;
				}
				set_available.set(lists);
			});
		}
	}

	pub fn add_to_list(&self, lid: String, oid: String, auth: crate::Auth) {
		let payload = apb::new()
			.set_activity_type(Some(apb::ActivityType::Add))
			.set_target(apb::Node::link(lid))
			.set_object(apb::Node::link(oid));

		leptos::task::spawn_local(async move {
			if let Err(e) = crate::Http::post(&auth.outbox(), &payload, auth).await {
				tracing::error!("error adding to list: {e}");
			}
		});
	}
}

impl Default for ListControls {
	fn default() -> Self {
		let (active, set_active) = signal(None);
		let (available, set_available) = signal(Vec::new());
		ListControls { active, set_active, available, set_available, }
	}
}
