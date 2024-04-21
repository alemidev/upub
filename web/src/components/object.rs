use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Base, Object};


#[component]
pub fn Object(object: serde_json::Value) -> impl IntoView {
	let oid = object.id().unwrap_or_default().to_string();
	let in_reply_to = object.in_reply_to().id().unwrap_or_default();
	let summary = object.summary().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get_or(&author_id, serde_json::Value::String(author_id.clone()));
	view! {
		<div>
			<table class="w-100 post-table pa-1 mb-s" >
				{move || if !in_reply_to.is_empty() {
					Some(view! {
						<tr class="post-table" >
							<td class="post-table pa-1" colspan="2" >
								"in reply to "<small><a class="clean hover" href={Uri::web(FetchKind::Object, &in_reply_to)}>{Uri::pretty(&in_reply_to)}</a></small>
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
					<td class="post-table pa-1" ><ActorBanner object=author /></td>
					<td class="post-table pa-1 center" >
						<a class="clean hover" href={oid} target="_blank">
							<DateTime t=object.published() />
							<PrivacyMarker addressed=object.addressed() />
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
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get_or(&author_id, serde_json::Value::String(author_id.clone()));
	view! {
		<table class="align w-100">
			<tr>
				<td><ActorBanner object=author /></td>
				<td class="rev" >
					<a class="clean hover" href={Uri::web(FetchKind::Object, object.id().unwrap_or_default())}>
						<DateTime t=object.published() />
					</a>
					<PrivacyMarker addressed=object.addressed() />
				</td>
			</tr>
		</table>
		<blockquote class="tl">
			{if summary.is_empty() { None } else { Some(view! { <code class="color ml-1">{summary}</code> })}}
			{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
		</blockquote>
	}
}

