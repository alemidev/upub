use leptos::*;
use crate::{prelude::*, URL_SENSITIVE};

use apb::{target::Addressed, Base, Object};

#[component]
pub fn Attachment(
	object: serde_json::Value,
	#[prop(optional)]
	sensitive: bool
) -> impl IntoView {
	let (expand, set_expand) = create_signal(false);
	let href = object.url().id().unwrap_or_default();
	let media_type = object.media_type()
		.unwrap_or("image/png") // TODO weird defaulting to png?????
		.to_string();
	let kind = media_type
		.split('/')
		.next()
		.unwrap_or("image")
		.to_string();

	match kind.as_str() {
		"image" =>
			view! {
				<p class="center">
					<img
						class="attachment ml-1"
						class:expand=expand
						src={move || if sensitive && !expand.get() {
							URL_SENSITIVE.to_string()
						} else {
							href.clone()
						}}
						title={object.name().unwrap_or_default().to_string()}
						on:click=move |_| set_expand.set(!expand.get())
					/>
				</p>
			}.into_view(),

		"video" =>
			view! {
				<p class="center cursor box ml-1"
					on:click=move |_| set_expand.set(!expand.get())
					title={object.name().unwrap_or_default().to_string()}
				>
					<video controls loop class="attachment" class:expand=expand >
						<source src={href.clone()} type={media_type} />
						<a href={href} target="_blank">audio clip</a>
					</video>
				</p>
			}.into_view(),

		"audio" =>
			view! {
				<p class="center">
					<audio controls class="w-100">
						<source src={href.clone()} type={media_type} />
						<a href={href} target="_blank">audio clip</a>
					</audio>
				</p>
			}.into_view(),

		_ => 
			view! {
				<p class="center box">
					<code class="cw color center">
						<a href={href} target="_blank">{media_type}</a>
					</code>
					<p class="tiny-text">
						<small>{object.name().unwrap_or_default().to_string()}</small>
					</p>
				</p>
			}.into_view(),
	}
}


#[component]
pub fn Object(object: serde_json::Value) -> impl IntoView {
	let uid = object.id().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get_or(&author_id, serde_json::Value::String(author_id.clone()));
	let sensitive = object.sensitive().unwrap_or_default();
	let attachments = object.attachment()
		.map(|x| view! { <Attachment object=x sensitive=sensitive /> })
		.collect_view();
	let attachments_padding = if object.attachment().is_empty() {
		None
	} else {
		Some(view! { <div class="pb-1"></div> })
	};
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
			<Summary summary=object.summary().map(|x| x.to_string()) open=!sensitive >
				{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
				{attachments_padding}
				{attachments}
			</Summary>
		</blockquote>
	}
}

#[component]
pub fn Summary(summary: Option<String>, open: bool, children: Children) -> impl IntoView {
	match summary.filter(|x| !x.is_empty()) {
		None => children().into_view(),
		Some(summary) => view! {
			<details class="pa-s" prop:open=open>
				<summary>
					<code class="cw color ml-s w-100">{summary}</code>
				</summary>
				{children()}
			</details>
		}.into_view(),
	}
}

