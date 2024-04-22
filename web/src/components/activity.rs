
use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Activity, Actor, Base, Object};


#[component]
pub fn ActivityLine(activity: serde_json::Value) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = CACHE.get_or(&actor_id, serde_json::Value::String(actor_id.clone()));
	let avatar = actor.icon().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
	let username = actor.preferred_username().unwrap_or_default().to_string();
	let domain = actor.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		<div>
			<a href={Uri::web(FetchKind::User, &actor_id)} class="clean hover">
				<img src={avatar} class="avatar-inline mr-s ml-1-r" /><b>{username}</b><small>@{domain}</small>
			</a>
			<span style="float:right">
				<a class="upub-title clean" href={Uri::web(FetchKind::Object, &object_id)} ><code class="color moreinfo" title={object_id.clone()} >{kind.as_ref().to_string()}</code></a>
				<a class="hover ml-1 hidden-on-mobile" href={Uri::web(FetchKind::Object, &object_id)} >
					<DateTime t=activity.published() />
				</a>
				<span class="hidden-on-mobile">
				<PrivacyMarker addressed=activity.addressed() />
				</span>
			</span>
		</div>
	}
}
