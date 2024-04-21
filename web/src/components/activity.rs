
use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Activity, Actor, Base, Object};


#[component]
pub fn ActivityLine(activity: serde_json::Value) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let object = CACHE.get(&object_id).unwrap_or(serde_json::Value::String(object_id.clone()));
	let addressed = activity.addressed();
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = match CACHE.get(&actor_id) {
		Some(a) => a,
		None => serde_json::Value::String(actor_id.clone()),
	};
	let avatar = actor.icon().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
	let username = actor.preferred_username().unwrap_or_default().to_string();
	let domain = actor.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		<div>
			<table class="align w-100" style="table-layout: fixed">
				<tr>
					<td>
						<a href={Uri::web("users", &actor_id)} class="clean hover">
							<img src={avatar} class="avatar-inline mr-s ml-1" /><b>{username}</b><small>@{domain}</small>
						</a>
					</td>
					<td class="rev" >
						<code class="color moreinfo" title={object_id.clone()} >{kind.as_ref().to_string()}</code>
						<a class="hover ml-1" href={Uri::web("objects", &object_id)} >
							<DateTime t=activity.published() />
						</a>
						<PrivacyMarker addressed=activity.addressed() />
					</td>
				</tr>
			</table>
		</div>
	}
}
