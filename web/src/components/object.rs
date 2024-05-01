use leptos::*;
use crate::{prelude::*, URL_SENSITIVE};

use apb::{target::Addressed, ActivityMut, Base, Collection, Object, ObjectMut};

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
				<div class="center cursor box ml-1"
					on:click=move |_| set_expand.set(!expand.get())
					title={object.name().unwrap_or_default().to_string()}
				>
					<video controls loop class="attachment" class:expand=expand >
						<source src={href.clone()} type={media_type} />
						<a href={href} target="_blank">audio clip</a>
					</video>
				</div>
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
pub fn Object(object: crate::Object) -> impl IntoView {
	let oid = object.id().unwrap_or_default().to_string();
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CACHE.get_or(&author_id, serde_json::Value::String(author_id.clone()).into());
	let sensitive = object.sensitive().unwrap_or_default();
	let addressed = object.addressed();
	let public = addressed.iter().any(|x| x.as_str() == apb::target::PUBLIC);
	let attachments = object.attachment()
		.map(|x| view! { <Attachment object=x sensitive=sensitive /> })
		.collect_view();
	let comments = object.replies().get()
		.map_or(0, |x| x.total_items().unwrap_or(0));
	let shares = object.generator().get()
		.map_or(0, |x| x.total_items().unwrap_or(0));
	let likes = object.audience().get()
		.map_or(0, |x| x.total_items().unwrap_or(0));
	let already_liked = object.audience().get()
		.map_or(false, |x| !x.ordered_items().is_empty()); // TODO check if contains my uid
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
					<PrivacyMarker addressed=addressed />
					<a class="clean hover ml-s" href={Uri::web(FetchKind::Object, object.id().unwrap_or_default())}>
						<DateTime t=object.published() />
					</a>
					<sup><small><a class="clean ml-s" href={oid.clone()} target="_blank">"↗"</a></small></sup>
				</td>
			</tr>
		</table>
		<blockquote class="tl">
			<Summary summary=object.summary().map(|x| x.to_string()) open=false >
				{content.into_iter().map(|x| view! { <p>{x}</p> }).collect_view()}
				{attachments_padding}
				{attachments}
			</Summary>
		</blockquote>
		<div class="mt-s ml-1 rev">
			<ReplyButton n=comments target=oid.clone() />
			<LikeButton n=likes liked=already_liked target=oid.clone() author=author_id private=!public />
			<RepostButton n=shares target=oid />
		</div>
	}
}

#[component]
pub fn Summary(summary: Option<String>, open: bool, children: Children) -> impl IntoView {
	match summary.filter(|x| !x.is_empty()) {
		None => children().into_view(),
		Some(summary) => view! {
			<details class="pa-s" prop:open=open>
				<summary>
					<code class="cw center color ml-s w-100">{summary}</code>
				</summary>
				{children()}
			</details>
		}.into_view(),
	}
}

#[component]
pub fn LikeButton(
	n: u64,
	target: String,
	liked: bool,
	author: String,
	#[prop(optional)]
	private: bool,
) -> impl IntoView {
	let (count, set_count) = create_signal(n);
	let (clicked, set_clicked) = create_signal(!liked);
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<span
			class:emoji=clicked
			class:emoji-btn=move || auth.present()
			class:cursor=move || clicked.get() && auth.present()
			class="ml-2"
			on:click=move |_ev| {
				if !auth.present() { return; }
				if !clicked.get() { return; }
				let to = apb::Node::links(vec![author.to_string()]);
				let cc = if private { apb::Node::Empty } else {
					apb::Node::links(vec![
						apb::target::PUBLIC.to_string(),
						format!("{URL_BASE}/users/{}/followers", auth.username())
					])
				};
				let payload = serde_json::Value::Object(serde_json::Map::default())
					.set_activity_type(Some(apb::ActivityType::Like))
					.set_object(apb::Node::link(target.clone()))
					.set_to(to)
					.set_cc(cc);
				spawn_local(async move {
					match Http::post(&auth.outbox(), &payload, auth).await {
						Ok(()) => {
							set_clicked.set(false);
							set_count.set(count.get() + 1);
						},
						Err(e) => tracing::error!("failed sending like: {e}"),
					}
				});
			}
		>
			{move || if count.get() > 0 { Some(view! { <small>{count}</small> })} else { None }}
			" ⭐"
		</span>
	}
}

#[component]
pub fn ReplyButton(n: u64, target: String) -> impl IntoView {
	let reply = use_context::<ReplyControls>().expect("missing reply controls context");
	let auth = use_context::<Auth>().expect("missing auth context");
	let comments = if n > 0 {
		Some(view! { <small>{n}</small> })
	} else {
		None
	};
	let _target = target.clone(); // TODO ughhhh useless clones
	view! {
		<span
			class:emoji=move || !reply.reply_to.get().map_or(false, |x| x == _target)
			// TODO can we merge these two classes conditions?
			class:emoji-btn=move || auth.present()
			class:cursor=move || auth.present()
			class="ml-2"
			on:click=move |_ev| if auth.present() { reply.reply(&target) }
		>
			{comments}
			" 📨"
		</span>
	}
}

#[component]
pub fn RepostButton(n: u64, target: String) -> impl IntoView {
	let (count, set_count) = create_signal(n);
	let (clicked, set_clicked) = create_signal(true);
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<span
			class:emoji=clicked
			class:emoji-btn=move || auth.present()
			class:cursor=move || clicked.get() && auth.present()
			class="ml-2"
			on:click=move |_ev| {
				if !auth.present() { return; }
				if !clicked.get() { return; }
				set_clicked.set(false);
				let to = apb::Node::links(vec![apb::target::PUBLIC.to_string()]);
				let cc = apb::Node::links(vec![format!("{URL_BASE}/users/{}/followers", auth.username())]);
				let payload = serde_json::Value::Object(serde_json::Map::default())
					.set_activity_type(Some(apb::ActivityType::Announce))
					.set_object(apb::Node::link(target.clone()))
					.set_to(to)
					.set_cc(cc);
				spawn_local(async move {
					match Http::post(&auth.outbox(), &payload, auth).await {
						Ok(()) => set_count.set(count.get() + 1),
						Err(e) => tracing::error!("failed sending like: {e}"),
					}
					set_clicked.set(true);
				});
			}
		>
			{move || if count.get() > 0 { Some(view! { <small>{count}</small> })} else { None }}
			" 🚀"
		</span>
	}
}
