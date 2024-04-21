use apb::ObjectMut;

use leptos::*;
use crate::prelude::*;

#[component]
pub fn Navigator() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<a href="/web/home"><input class="w-100" type="submit" class:hidden=move || !auth.present() value="home timeline" /></a>
		<a href="/web/server"><input class="w-100" type="submit" value="server timeline" /></a>
		<a href="/web/about"><input class="w-100" type="submit" value="about" /></a>
		<a href="/web/debug"><input class="w-100" type="submit" value="debug" class:hidden=move|| !auth.present() /></a>
	}
}

#[component]
pub fn PostBox(username: Signal<Option<String>>) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let content_ref: NodeRef<html::Textarea> = create_node_ref();
	let public_ref: NodeRef<html::Input> = create_node_ref();
	let followers_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div class:hidden=move || !auth.present() >
			<table class="align w-100">
				<tr>
				<td><input type="checkbox" title="public" value="public" node_ref=public_ref /></td>
				<td class="w-100"><input class="w-100" type="text" node_ref=summary_ref title="summary" /></td>
				<td><input type="checkbox" title="followers" value="followers" node_ref=followers_ref checked /></td>
				</tr>
				<tr>
					<td colspan="3">
						<textarea rows="5" class="w-100" node_ref=content_ref title="content" ></textarea>
					</td>
				</tr>
				<tr>
					<td colspan="3">
						<button class="w-100" type="button" on:click=move |_| {
							spawn_local(async move {
								let summary = summary_ref.get().map(|x| x.value());
								let content = content_ref.get().map(|x| x.value()).unwrap_or_default();
								let public = public_ref.get().map(|x| x.checked()).unwrap_or_default();
								let followers = followers_ref.get().map(|x| x.checked()).unwrap_or_default();
								match Http::post(
									&format!("{URL_BASE}/users/test/outbox"),
									&serde_json::Value::Object(serde_json::Map::default())
										.set_object_type(Some(apb::ObjectType::Note))
										.set_summary(summary.as_deref())
										.set_content(Some(&content))
										.set_to(
											if public {
												apb::Node::links(vec![apb::target::PUBLIC.to_string()])
											} else { apb::Node::Empty }
										)
										.set_cc(
											if followers {
												apb::Node::links(vec![format!("{URL_BASE}/users/{}/followers", username.get().unwrap_or_default())])
											} else { apb::Node::Empty }
										),
									auth
								)
									.await
								{
									Err(e) => tracing::error!("error posting note: {e}"),
									Ok(()) => {
										if let Some(x) = summary_ref.get() { x.set_value("") }
										if let Some(x) = content_ref.get() { x.set_value("") }
									},
								}
							})
						} >post</button>
					</td>
				</tr>
			</table>
		</div>
	}
}

#[component]
pub fn Breadcrumb(
	#[prop(optional)]
	back: bool,
	children: Children,
) -> impl IntoView {
	view! {
		<div class="tl-header w-100 center" >
			{if back { Some(view! {
				<a class="breadcrumb mr-1" href="javascript:history.back()" ><b>"<<"</b></a>
			})} else { None }}
			<b>{crate::NAME}</b>" :: "{children()}
		</div>
	}
}
