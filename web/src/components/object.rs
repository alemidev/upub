use leptos::*;
use crate::prelude::*;

use apb::{target::Addressed, Base, Object};


#[component]
pub fn Object(object: serde_json::Value) -> impl IntoView {
	let uid = object.id().unwrap_or_default().to_string();
	let summary = object.summary().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get_or(&author_id, serde_json::Value::String(author_id.clone()));
	let attachments = object.attachment()
		.map(|x| view! {
			<p class="center"><img class="attachment" src={x.url().id().unwrap_or_default()} title={x.name().unwrap_or_default().to_string()} /></p>
		})
		.collect_view();
	view! {
		<table class="align w-100">
			<tr>
				<td><ActorBanner object=author /></td>
				<td class="rev" >
					{object.in_reply_to().id().map(|reply| view! {
							<small><i><a class="clean" href={Uri::web(FetchKind::Object, &reply)} title={reply}>reply</a></i></small> 
					})}
					<PrivacyMarker addressed=object.addressed() />
					<a class="clean hover ml-s" href={Uri::web(FetchKind::Object, object.id().unwrap_or_default())}>
						<DateTime t=object.published() />
					</a>
					<sup><small><a class="clean ml-s" href={uid} target="_blank">"↗"</a></small></sup>
				</td>
			</tr>
		</table>
		<blockquote class="tl">
			{if summary.is_empty() { None } else { Some(view! { <code class="color ml-1">{summary}</code> })}}
			{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
		</blockquote>
		{attachments}
	}
}

