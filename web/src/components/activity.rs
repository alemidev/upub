use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Base, Activity, Object};


#[component]
pub fn ActivityLine(activity: crate::Object) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let activity_url = activity.id().map(|x| view! {
		<sup><small><a class="clean ml-s" href={x.to_string()} target="_blank">"↗"</a></small></sup>
	});
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = CACHE.get_or(&actor_id, serde_json::Value::String(actor_id.clone()).into());
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	let href = match kind {
		apb::ActivityType::Follow => Uri::web(U::Actor, &object_id),
		// TODO for update check what's being updated
		_ => Uri::web(U::Object, &object_id),
	};
	view! {
		<div>
			<span class="ml-1-r">
				<ActorStrip object=actor />
			</span>
			<span style="float:right">
				<code class="color moreinfo" title={activity.published().map(|x| x.to_rfc2822())} >
					<a class="upub-title clean" title={object_id} href={href} >
						{kind.as_ref().to_string()}
					</a>
					{activity_url}
					<PrivacyMarker addressed=activity.addressed() />
				</code>
			</span>
		</div>
	}
}

#[component]
pub fn Item(
	item: crate::Object,
	#[prop(optional)] sep: bool,
	#[prop(optional)] replies: bool,
) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let id = item.id().unwrap_or_default().to_string();
	let sep = if sep { Some(view! { <hr /> }) } else { None };
	if !replies && !config.get().filters.visible(&item) {
		return None;
	}
	match item.object_type().unwrap_or(apb::ObjectType::Object) {
		// special case for placeholder activities
		apb::ObjectType::Note | apb::ObjectType::Document(_) =>
			Some(view! { <Object object=item.clone() />{sep.clone()} }.into_view()),
		// everything else
		apb::ObjectType::Activity(t) => {
			let object_id = item.object().id().unwrap_or_default();
			let object = match t {
				apb::ActivityType::Create | apb::ActivityType::Announce => 
					CACHE.get(&object_id).map(|obj| {
						view! { <Object object=obj /> }
					}.into_view()),
				apb::ActivityType::Follow =>
					CACHE.get(&object_id).map(|obj| {
						view! {
							<div class="ml-1">
								<ActorBanner object=obj />
								<FollowRequestButtons activity_id=id.clone() actor_id=object_id />
							</div>
						}
					}.into_view()),
				_ => None,
			};
			Some(view! {
				<ActivityLine activity=item.clone() />
				{object}
				{sep.clone()}
			}.into_view())
		},
		// should never happen
		t => Some(view! { <p><code>type not implemented : {t.as_ref().to_string()}</code></p> }.into_view()),
	}
}
