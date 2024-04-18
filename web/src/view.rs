use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Activity, Actor, Base, Object};


#[component]
pub fn InlineActivity(activity: serde_json::Value) -> impl IntoView {
	let object_id = activity.object().id().unwrap_or_default();
	let object = CACHE.get(&object_id).unwrap_or(serde_json::Value::String(object_id.clone()));
	let addressed = activity.addressed();
	let audience = format!("[ {} ]", addressed.join(", "));
	let actor_id = activity.actor().id().unwrap_or_default();
	let actor = match CACHE.get(&actor_id) {
		Some(a) => a,
		None => serde_json::Value::String(actor_id.clone()),
	};
	let privacy = if addressed.iter().any(|x| x == apb::target::PUBLIC) {
		"🌐"
	} else if addressed.iter().any(|x| x.ends_with("/followers")) {
		"🔒"
	} else {
		"🔗"
	};
	let date = object.published().or(activity.published());
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		<div>
			<table class="align w-100" >
			<tr>
			<td rowspan="2" >
				<ActorBanner object=actor tiny=true />
			</td>
			<td class="rev" >
				<code class="color moreinfo" title={object_id.clone()} >{kind.as_ref().to_string()}</code>
				<span class="emoji ml-s moreinfo" title={audience} >{privacy}</span>
			</td>
		</tr>
		<tr>
			<td class="rev">
				<a class="hover" href={Uri::web("objects", &object_id)} >
					<DateTime t=date />
				</a>
			</td>
		</tr>
		</table>
		</div>
		{match kind {
			// post
			apb::ActivityType::Create => view! { <ObjectInline object=object /> }.into_view(),
			_ => view! {}.into_view(),
		}}
	}
}

#[component]
pub fn ActorBanner(
	object: serde_json::Value,
	#[prop(optional)]
	tiny: bool
) -> impl IntoView {
	match object {
		serde_json::Value::String(id) => view! {
			<div><b>?</b>" "<a class="clean hover" href={Uri::web("users", &id)}>{Uri::pretty(&id)}</a></div>
		},
		serde_json::Value::Object(_) => {
			let uid = object.id().unwrap_or_default().to_string();
			let uri = Uri::web("users", &uid);
			let avatar_url = object.icon().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
			let display_name = object.name().unwrap_or_default().to_string();
			let username = object.preferred_username().unwrap_or_default().to_string();
			let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
			view! {
				<div>
					<table class="align" >
					<tr>
						<td rowspan="2" ><a href={uri.clone()} ><img class="avatar-circle" class:inline-avatar=move|| tiny src={avatar_url} /></a></td>
						<td><b>{display_name}</b></td>
					</tr>
					<tr>
						<td class="top" ><a class="hover" href={uri} ><small>{username}@{domain}</small></a></td>
					</tr>
					</table>
				</div>
			}
		},
		_ => view! {
			<div><b>invalid actor</b></div>
		}
	}
}

#[component]
pub fn Object(object: serde_json::Value) -> impl IntoView {
	let oid = object.id().unwrap_or_default().to_string();
	let in_reply_to = object.in_reply_to().id().unwrap_or_default();
	let summary = object.summary().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let date = object.published();
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get(&author_id).unwrap_or(serde_json::Value::String(author_id.clone()));
	view! {
		<div>
			<table class="w-100 post-table pa-1 mb-s" >
				{move || if !in_reply_to.is_empty() {
					Some(view! {
						<tr class="post-table" >
							<td class="post-table pa-1" colspan="2" >
								"in reply to "<small><a class="clean hover" href={Uri::web("objects", &in_reply_to)}>{Uri::pretty(&in_reply_to)}</a></small>
							</td>
						</tr>
					})
				} else { None }}
				{move || if !summary.is_empty() {
					Some(view! {
						<tr class="post-table" >
							<td class="post-table pa-1" colspan="2" >{summary.clone()}</td>
						</tr>
					})
				} else { None }}
				<tr class="post-table" >
					<td class="post-table pa-1" colspan="2" >{
						content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()
					}</td>
				</tr>
				<tr class="post-table" >
					<td class="post-table pa-1" ><ActorBanner object=author tiny=true /></td>
					<td class="post-table pa-1 center" >
						<a class="clean hover" href={oid} target="_blank">
							<DateTime t=date />
						</a>
					</td>
				</tr>
			</table>
		</div>
	}
}

#[component]
pub fn ObjectInline(object: serde_json::Value) -> impl IntoView {
	let summary = object.summary().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	view! {
		{if summary.is_empty() { None } else { Some(view! { <code class="color">{summary}</code> })}}
		<blockquote class="tl">
			{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
		</blockquote>
	}
}

#[component]
pub fn DateTime(t: Option<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
	match t {
		Some(t) => {
			let pretty = t.format("%Y/%m/%d %H:%M:%S").to_string();
			let rfc = t.to_rfc3339();
			Some(view! {
				<small title={rfc}>{pretty}</small>
			})
		},
		None => None,
	}
}
