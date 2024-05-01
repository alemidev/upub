use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Activity, Object};


#[component]
pub fn ActivityLine(activity: crate::Object) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = CACHE.get_or(&actor_id, serde_json::Value::String(actor_id.clone()).into());
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		<div>
			<span class="ml-1-r">
				<ActorStrip object=actor />
			</span>
			<span style="float:right">
				<code class="color moreinfo" title={activity.published().map(|x| x.to_rfc2822())} >
					<a class="upub-title clean" title={object_id.clone()} href={Uri::web(FetchKind::Object, &object_id)} >
						{kind.as_ref().to_string()}
					</a>
					<PrivacyMarker addressed=activity.addressed() />
				</code>
			</span>
		</div>
	}
}
