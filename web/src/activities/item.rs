use leptos::*;
use crate::prelude::*;

use apb::{Activity, ActivityMut, Base, Object};


#[component]
pub fn ActivityLine(activity: crate::Object, children: Children) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let to = activity.to().all_ids();
	let cc = activity.cc().all_ids();
	let privacy = Privacy::from_addressed(&to, &cc);
	let activity_url = activity.id().map(|x| view! {
		<sup><small><a class="clean ml-s" href={x.to_string()} target="_blank">"↗"</a></small></sup>
	});
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = cache::OBJECTS.get_or(&actor_id, serde_json::Value::String(actor_id.clone()).into());
	let content = activity.content().unwrap_or_default();
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	let href = match kind {
		apb::ActivityType::Follow => Uri::web(U::Actor, &object_id),
		// TODO for update check what's being updated
		_ => Uri::web(U::Object, &object_id),
	};
	view! {
		<table class="align w-100">
			<tr>
				<td class="ml-1-r">
					<ActorStrip object=actor />
				</td>
				<td class="rev">
					{content}" "
					<code class="color" title={activity.published().ok().map(|x| x.to_rfc2822())} >
						{children()}
						<a class="upub-title clean" title={object_id} href={href} >
							{kind.as_ref().to_string()}
						</a>
						{activity_url}
						<PrivacyMarker privacy=privacy to=&to cc=&cc />
					</code>
				</td>
			</tr>
		</table>
	}
}

#[component]
pub fn Item(
	item: crate::Object,
	#[prop(optional)] sep: bool,
	#[prop(optional)] slim: bool,
	#[prop(optional)] always: bool,
) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let id = item.id().unwrap_or_default().to_string();
	let sep = if sep { Some(view! { <hr /> }) } else { None };
	let seen = item.seen().unwrap_or(true);
	move || {
		if !always && !config.get().filters.visible(&item) {
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
						cache::OBJECTS.get(&object_id).map(|obj| {
							view! { <Object object=obj /> }
						}.into_view()),
					apb::ActivityType::Follow =>
						cache::OBJECTS.get(&object_id).map(|obj| {
							view! {
								<div class="ml-1">
									<ActorBanner object=obj />
									<FollowRequestButtons activity_id=id.clone() actor_id=object_id />
								</div>
							}
						}.into_view()),
					_ => None,
				};
				if !seen {
					let (not_seen, not_seen_tx) = create_signal(!seen);
					let id = id.clone();
					Some(view! {
						<div class:notification=not_seen>
							{if !slim { Some(view! {
								<ActivityLine activity=item.clone() >
									{move || if not_seen.get() { Some(view! { <AckBtn id=id.clone() tx=not_seen_tx /> }) } else { None }}
								</ActivityLine>
							}) } else {
								None
							}}
							{object}
						</div>
						{sep.clone()}
					}.into_view())
				} else {
					Some(view! {
						<div>
							{if !slim { Some(view! { <ActivityLine activity=item.clone() >""</ActivityLine> }) } else { None }}
							{object}
						</div>
						{sep.clone()}
					}.into_view())
				}
			},
			// should never happen
			t => Some(view! { <p><code>type not implemented : {t.as_ref().to_string()}</code></p> }.into_view()),
		}
	}
}

#[component]
fn AckBtn(id: String, tx: WriteSignal<bool>) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let (notifications, set_notifications) = use_context::<(ReadSignal<u64>, WriteSignal<u64>)>().expect("missing notifications context");

	view! {
		<span
			class="emoji emoji-btn cursor mr-1"
			on:click=move |e| {
				e.prevent_default();
				let payload = apb::new()
					.set_activity_type(Some(apb::ActivityType::View))
					.set_object(apb::Node::link(id.clone()));
				let id = id.clone();
				spawn_local(async move {
					if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
						tracing::error!("failed marking notification as seen: {e}");
					} else {
						tx.set(false);
						set_notifications.set(notifications.get() - 1);
						if let Some(activity) = crate::cache::OBJECTS.get(&id) {
							let changed = (*activity).clone().set_seen(Some(true));
							crate::cache::OBJECTS.store(&id, std::sync::Arc::new(changed));
						}
					}
				});
			}
		>
			"✔️"
		</span>
	}
}
