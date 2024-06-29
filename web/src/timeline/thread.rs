use apb::{field::OptionalString, Activity, Base, Object};
use leptos::*;
use crate::prelude::*;
use super::Timeline;

#[component]
pub fn Thread(tl: Timeline, root: String) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = leptos::watch(
			move || auto_scroll.get(),
			move |new, old, _| {
				match old {
					None => tl.spawn_more(auth), // always do it first time
					Some(old) => if *new && new != old {
						tl.spawn_more(auth);
					},
				}
			},
			true,
		);
	}

	view! {
		<div>
			<FeedRecursive tl=tl root=root />
		</div>
		{move || if tl.loading.get() { Some(view! { <Loader /> }) } else { None }}
	}
}

#[component]
fn FeedRecursive(tl: Timeline, root: String) -> impl IntoView {
	let root_values = move || tl.feed
		.get()
		.into_iter()
		.filter_map(|x| {
			let document = CACHE.get(&x)?;
			let (oid, reply) = match document.object_type().ok()? {
				// if it's a create, get and check created object: does it reply to root?
				apb::ObjectType::Activity(apb::ActivityType::Create) => {
					let object = CACHE.get(document.object().id().ok()?)?;
					(object.id().str()?, object.in_reply_to().id().str()?)
				},

				// if it's a raw note, directly check if it replies to root
				apb::ObjectType::Note => (document.id().str()?, document.in_reply_to().id().str()?),

				// if it's anything else, check if it relates to root, maybe like or announce?
				_ => (document.id().str()?, document.object().id().str()?),
			};
			if reply == root {
				Some((oid, document))
			} else {
				None
			}
		})
		.collect::<Vec<(String, crate::Object)>>();

	view! {
		<For
			each=root_values
			key=|(id, _obj)| id.clone()
			children=move |(id, obj)|
				view! {
					<div class="context depth-r">
						<Item item=obj replies=true slim=true />
						<div class="depth-r">
							<FeedRecursive tl=tl root=id />
						</div>
					</div>
				}
		/ >
	}
}
