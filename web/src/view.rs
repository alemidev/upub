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
	let date = object.published().map(|x| x.format("%Y/%m/%d %H:%M:%S").to_string()).unwrap_or_else(||
		activity.published().map(|x| x.format("%Y/%m/%d %H:%M:%S").to_string()).unwrap_or_default()
	);
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		<div>
			<table class="align w-100" >
			<tr>
			<td rowspan="2" >
				<ActorBanner object=actor />
			</td>
			<td class="rev" >
				<code class="color moreinfo" title={object_id.clone()} >{kind.as_ref().to_string()}</code>
				<span class="emoji ml-s moreinfo" title={audience} >{privacy}</span>
			</td>
		</tr>
		<tr>
			<td class="rev">
				<a class="hover" href={Uri::web("objects", &object_id)} >
					<small>{date}</small>
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
pub fn ActorBanner(object: serde_json::Value) -> impl IntoView {
	match object {
		serde_json::Value::String(id) => view! {
			<div><b>{id}</b></div>
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
						<td rowspan="2" ><a href={uri.clone()} ><img class="avatar-circle" src={avatar_url} /></a></td>
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
	let summary = object.summary().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let date = object.published().map(|x| x.format("%Y/%m/%d %H:%M:%S").to_string()).unwrap_or_default();
	let date_rfc = object.published().map(|x| x.to_rfc3339()).unwrap_or_default();
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get(&author_id).unwrap_or(serde_json::Value::String(author_id.clone()));
	view! {
		<div>
			<table class="w-100 post-table pa-1 mb-s" >
				{move || if !summary.is_empty() {
					view! {
						<tr class="post-table" >
							<td class="post-table pa-1" colspan="2" >{summary.clone()}</td>
						</tr>
					}.into_view()
				} else {
					view! { }.into_view()
				}}
				<tr class="post-table" >
					<td class="post-table pa-1" colspan="2" >{
						content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()
					}</td>
				</tr>
				<tr class="post-table" >
					<td class="post-table pa-1" ><ActorBanner object=author /></td>
					<td class="post-table pa-1 center" ><small title={date_rfc} >{date}</small></td>
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
		<blockquote>
			{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
		</blockquote>
	}
}
