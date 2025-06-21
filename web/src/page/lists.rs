use apb::{ActivityMut, CollectionMut, ObjectMut};
use leptos::prelude::*;
use crate::prelude::*;

#[component]
pub fn ListsPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let uid = auth.username();

	let (error, set_error) = signal(None);

	let name_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let summary_ref: NodeRef<leptos::html::Input> = NodeRef::new();

	view! {
		<Loadable
			base=format!("{URL_BASE}/actors/{uid}/lists/page")
			element=move |obj| view! { <List list=obj /> }
		/>

		<blockquote class="mt-3">
			<details class="cw">
				<summary>
					<code class="cw center color">new list</code>
				</summary>
					<table class="align w-100">
						<tr>
							<td class="w-33"><input class="w-100" type="text" node_ref=name_ref placeholder="name" required /></td>
							<td class="w-50"><input class="w-100" type="text" node_ref=summary_ref placeholder="summary" /></td>
							<td class="w-33">
								<input class="w-100" type="submit" value="create" on:click=move |ev| {
									ev.prevent_default();
									let name = name_ref.get().map(|x| x.value()).filter(|x| !x.is_empty());
									if name.is_none() {
										set_error.set(Some("'name' is required".to_string()));
										return;
									}
									let summary = summary_ref.get().map(|x| x.value()).filter(|x| !x.is_empty());
									let payload = apb::new()
										.set_activity_type(Some(apb::ActivityType::Create))
										.set_object(apb::Node::object(
											apb::new()
												.set_collection_type(Some(apb::CollectionType::Collection))
												.set_name(name)
												.set_summary(summary)
										));
									leptos::task::spawn_local(async move {
										match crate::Http::post(&auth.outbox(), &payload, auth).await {
											Ok(()) => {
												if let Some(name_ref_element) = name_ref.get() {
													name_ref_element.set_value("");
												}
												if let Some(summary_ref_element) = summary_ref.get() {
													summary_ref_element.set_value("");
												}
												set_error.set(None);
											},
											Err(e) => {
												tracing::error!("{e}");
												set_error.set(Some(e.to_string()));
											},
										}
									});
								} />
							</td>
						</tr>
						<tr>
							<td colspan="3" class="center"><b>{error}</b></td>
						</tr>
					</table>
			</details>
		</blockquote>
	}
}
