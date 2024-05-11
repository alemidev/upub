use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Base, Activity, Object};


#[component]
pub fn ActivityLine(activity: crate::Object) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = CACHE.get_or(&actor_id, serde_json::Value::String(actor_id.clone()).into());
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	let href = match kind {
		apb::ActivityType::Follow => Uri::web(FetchKind::User, &object_id),
		// TODO for update check what's being updated
		_ => Uri::web(FetchKind::Object, &object_id),
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
					<PrivacyMarker addressed=activity.addressed() />
				</code>
			</span>
		</div>
	}
}

#[component]
pub fn Item(item: crate::Object) -> impl IntoView {
	let id = item.id().unwrap_or_default().to_string();
	match item.object_type() {
		// special case for placeholder activities
		Some(apb::ObjectType::Note) | Some(apb::ObjectType::Document(_)) =>
			view! { <Object object=item /> }.into_view(),
		// everything else
		Some(apb::ObjectType::Activity(t)) => {
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
								<FollowRequestButtons activity_id=id actor_id=object_id />
							</div>
						}
					}.into_view()),
				_ => None,
			};
			view! {
				<ActivityLine activity=item />
				{object}
			}.into_view()
		},
		// should never happen
		_ => view! { <p><code>type not implemented</code></p> }.into_view(),
	}
}
