use leptos::*;
use crate::{prelude::*, URL_SENSITIVE};

use base64::prelude::*;
use apb::{field::OptionalString, Document, Object};

fn uncloak(txt: Option<&str>) -> Option<String> {
	let decoded = BASE64_URL_SAFE.decode(txt?).ok()?;
	Some(std::str::from_utf8(&decoded).ok()?.to_string())
}

#[component]
pub fn Attachment(
	object: serde_json::Value,
	#[prop(optional)]
	sensitive: bool
) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let (expand, set_expand) = create_signal(false);
	let href = object.url().id().str().unwrap_or_default();
	let uncloaked = uncloak(href.split('/').last()).unwrap_or_default();
	let media_type = object.media_type()
		.unwrap_or("link") // TODO make it an Option rather than defaulting to link everywhere
		.to_string();
	let mut kind = media_type
		.split('/')
		.next()
		.unwrap_or("link")
		.to_string();

	// TODO in theory we should match on document_type, but mastodon and misskey send all attachments
	// as "Documents" regardless of type, so we're forced to ignore the actual AP type and just match
	// using media_type, uffff
	//
	// those who correctly send Image type objects without a media type get shown as links here, this
	// is a dirty fix to properly display as images
	if kind == "link" && matches!(object.document_type(), Ok(apb::DocumentType::Image)) {
		kind = "image".to_string();
	}

	match kind.as_str() {
		"image" =>
			view! {
				<p class="center">
					<img
						class="w-100 attachment"
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

		"video" => {
			let _href = href.clone();
			view! {
				<div class="center cursor box ml-1"
					on:click=move |_| set_expand.set(!expand.get())
					title={object.name().unwrap_or_default().to_string()}
				>
					<video controls class="attachment" class:expand=expand prop:loop=move || config.get().loop_videos  >
						{move || if sensitive && !expand.get() { None } else { Some(view! { <source src={_href.clone()} type={media_type.clone()} /> }) }}
						<a href={href.clone()} target="_blank">video clip</a>
					</video>
				</div>
			}.into_view()
		},

		"audio" =>
			view! {
				<p class="center">
					<audio controls class="w-100" prop:loop=move || config.get().loop_videos >
						<source src={href.clone()} type={media_type} />
						<a href={href} target="_blank">audio clip</a>
					</audio>
				</p>
			}.into_view(),

		"link" | "text" =>
			view! {
				<p class="mt-s mb-s">
					<a title={uncloaked.clone()} href={uncloaked.clone()} rel="noreferrer nofollow" target="_blank">
						{Uri::pretty(&uncloaked, 50)}
					</a>
				</p>
			}.into_view(),

		_ => 
			view! {
				<p class="center box">
					<code class="cw color center">
						<a href={href} target="_blank">{media_type}</a>
					</code>
					{object.name().map(|name| {
						view! { <p class="tiny-text"><small>{name.to_string()}</small></p> }
					})}
				</p>
			}.into_view(),
	}
}


