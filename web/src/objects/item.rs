use std::sync::Arc;

use leptos::*;
use crate::{prelude::*, URL_SENSITIVE};

use apb::{ActivityMut, Base, Collection, CollectionMut, Object, ObjectMut, Shortcuts};

#[component]
pub fn Object(object: crate::Object) -> impl IntoView {
	let oid = object.id().unwrap_or_default().to_string();
	let author_id = object.attributed_to().id().ok().unwrap_or_default();
	let author = cache::OBJECTS.get_or(&author_id, serde_json::Value::String(author_id.clone()).into());
	let sensitive = object.sensitive().unwrap_or_default();
	let to = object.to().all_ids();
	let cc = object.cc().all_ids();
	let privacy = Privacy::from_addressed(&to, &cc);
	let external_url = object.url().id().ok().unwrap_or_else(|| oid.clone());
	let attachments = object.attachment()
		.flat()
		.into_iter()
		.filter_map(|x| x.into_inner().ok()) // TODO maybe show links?
		.map(|x| view! { <Attachment object=x sensitive=sensitive /> })
		.collect_view();
	let comments = object.replies_count().unwrap_or_default();
	let shares = object.shares_count().unwrap_or_default();
	let likes = object.likes_count().unwrap_or_default();
	let already_liked = object.liked_by_me().unwrap_or(false);

	let attachments_padding = if object.attachment().is_empty() {
		None
	} else {
		Some(view! { <div class="pb-1"></div> })
	};

	let content = mdhtml::safe_html(&object.content().unwrap_or_default());

	let audience_badge = object.audience().id().ok()
		.map(|x| {
			// TODO this isn't guaranteed to work every time...
			let name = x.split('/').last().unwrap_or_default().to_string();
			view! {
				<a class="clean dim" href={Uri::web(U::Actor, &x)}>
					<span class="border-button ml-s" title={x}>
						<code class="color mr-s">&</code>
						<small class="mr-s">
							{name}
						</small>
					</span>
				</a>
			}
		});

	let quote_block = object.quote_url()
		.id()
		.ok()
		.and_then(|x| {
			Some(view! {
				<div class="quote">
					<Object object=crate::cache::OBJECTS.get(&x)? />
				</div>
			})
		});

	let quote_badge = object.quote_url()
		.id()
		.map(|x| {
			let href = Uri::web(U::Object, &x);
			view! {
				<a class="clean dim" href={href}>
					<span class="border-button ml-s" >
						<code class="color mr-s">">"</code>
						<small class="mr-s">
							quote
						</small>
					</span>
				</a>" "
			}
		})
		.ok();

	let tag_badges = object.tag()
		.flat()
		.into_iter()
		.filter_map(|node| match node {
			apb::Node::Link(x) => Some(x),
			_ => None,
		})
		.map(|link| {
			match apb::Link::link_type(link.as_ref()) {
				Ok(apb::LinkType::Hashtag) => {
					let name = apb::Link::name(link.as_ref()).unwrap_or_default().replace('#', "");
					let href = Uri::web(U::Hashtag, &name);
					Some(view! {
						<a class="clean dim" href={href}>
							<span class="border-button ml-s" >
								<code class="color mr-s">#</code>
								<small class="mr-s">
									{name}
								</small>
							</span>
						</a>" "
					})
				},
				Ok(apb::LinkType::Mention) => {
					let uid = apb::Link::href(link.as_ref()).unwrap_or_default();
					let mention = apb::Link::name(link.as_ref()).unwrap_or_default().replacen('@', "", 1);
					let (username, domain) = if let Some((username, server)) = mention.split_once('@') {
						(username.to_string(), server.to_string())
					} else {
						(
							mention.to_string(),
							uid.replace("https://", "").replace("http://", "").split('/').next().unwrap_or_default().to_string(),
						)
					};
					let href = Uri::web(U::Actor, &uid);
					Some(view! {
						<a class="clean dim" href={href}>
							<span class="border-button ml-s" title={format!("@{username}@{domain}")} >
								<code class="color mr-s">@</code>
								<small class="mr-s">
									{username}
								</small>
							</span>
						</a>" "
					})
				},
				_ => None,
			}
		}).collect_view();

	let post_image = object.image().inner().and_then(|x| x.url().id()).ok().map(|x| {
		let (expand, set_expand) = create_signal(false);
		view! {
			<img
				class="flex-pic box cursor"
				class:flex-pic-expand=expand
				src={move || if sensitive && !expand.get() {
					URL_SENSITIVE.to_string()
				} else {
					x.clone()
				}}
				on:click=move|_| set_expand.set(!expand.get())
			/>
		}
	});

	let post_inner = view! {
		<Summary summary=object.summary().ok().map(|x| x.to_string()) >
			<p inner_html={content}></p>
			{attachments_padding}
			{attachments}
		</Summary>
	};
	let post = match object.object_type() {
		// mastodon, pleroma, misskey
		Ok(apb::ObjectType::Note) => view! {
			<article class="tl">
				{post_inner}
				{quote_block}
			</article>
		}.into_view(),
		// lemmy with Page, peertube with Video
		Ok(apb::ObjectType::Document(t)) => view! {
			<article class="float-container ml-1 mr-1" >
				{post_image}
				<div>
					<h4 class="mt-s mb-1" title={t.as_ref().to_string()}>
						<b>{object.name().unwrap_or_default().to_string()}</b>
					</h4>
					{post_inner}
					{quote_block}
				</div>
			</article>
		}.into_view(),
		// wordpress, ... ?
		Ok(apb::ObjectType::Article) => view! {
			<article>
				<h3>{object.name().unwrap_or_default().to_string()}</h3>
				<hr />
				{post_inner}
				{quote_block}
			</article>
		}.into_view(),
		// everything else
		Ok(t) => view! {
			<h3>{t.as_ref().to_string()}</h3>
			{post_inner}
			{quote_block}
		}.into_view(),
		// object without type?
		Err(_) => view! { <code>missing object type</code> }.into_view(),
	};
	view! {
		<table class="align w-100 ml-s mr-s">
			<tr>
				<td><ActorBanner object=author /></td>
				<td class="rev" >
					{object.in_reply_to().id().ok().map(|reply| view! {
							<small><i><a class="clean" href={Uri::web(U::Object, &reply)} title={reply}>reply</a></i></small> 
					})}
					<PrivacyMarker privacy=privacy to=&to cc=&cc />
					<a class="clean hover ml-s" href={Uri::web(U::Object, &object.id().unwrap_or_default())}>
						<DateTime t=object.published().ok() />
					</a>
					<sup><small><a class="clean ml-s" href={external_url} target="_blank">"↗"</a></small></sup>
				</td>
			</tr>
		</table>
		{post}
		<div class="mb-s mt-s ml-1 rev">
			{quote_badge}
			{tag_badges}
			{audience_badge}
			<span style="white-space:nowrap">
				<ReplyButton n=comments target=oid.clone() />
				<LikeButton n=likes liked=already_liked target=oid.clone() author=author_id.clone() private=!privacy.is_public() />
				{if privacy.is_public() { Some(view! { <RepostButton n=shares target=oid author=author_id /> }) } else { None }}
			</span>
		</div>
	}
}

#[component]
pub fn Summary(summary: Option<String>, children: Children) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	match summary.filter(|x| !x.is_empty()) {
		None => children().into_view(),
		Some(summary) => view! {
			<details class="pa-s" prop:open=move || !config.get().collapse_content_warnings>
				<summary>
					<code class="cw center color ml-s w-100 bb">{summary}</code>
				</summary>
				{children()}
			</details>
		}.into_view(),
	}
}

#[component]
pub fn LikeButton(
	n: i32,
	target: String,
	liked: bool,
	author: String,
	#[prop(optional)]
	private: bool,
) -> impl IntoView {
	let (count, set_count) = create_signal(n);
	let (clicked, set_clicked) = create_signal(!liked);
	let auth = use_context::<Auth>().expect("missing auth context");
	let privacy = use_context::<PrivacyControl>().expect("missing privacy context");
	view! {
		<span
			class:emoji=clicked
			class:emoji-btn=move || auth.present()
			class:cursor=move || clicked.get() && auth.present()
			class="ml-2"
			on:click=move |_ev| {
				if !auth.present() { return; }
				if !clicked.get() { return; }
				let (mut to, cc) = if private {
					(vec![], vec![])
				} else {
					privacy.get().address(&auth.username())
				};
				to.push(author.clone());
				let payload = serde_json::Value::Object(serde_json::Map::default())
					.set_activity_type(Some(apb::ActivityType::Like))
					.set_object(apb::Node::link(target.clone()))
					.set_to(apb::Node::links(to))
					.set_cc(apb::Node::links(cc));
				let target = target.clone();
				spawn_local(async move {
					match Http::post(&auth.outbox(), &payload, auth).await {
						Ok(()) => {
							set_clicked.set(false);
							set_count.set(count.get() + 1);
							if let Some(cached) = cache::OBJECTS.get(&target) {
								let mut new = (*cached).clone().set_liked_by_me(Some(true));
								if let Ok(likes) = new.likes().inner() {
									if let Ok(count) = likes.total_items() {
										new = new.set_likes(apb::Node::object(likes.clone().set_total_items(Some(count + 1))));
									}
								}
								cache::OBJECTS.store(&target, Arc::new(new));
							}
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
pub fn ReplyButton(n: i32, target: String) -> impl IntoView {
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
pub fn RepostButton(n: i32, target: String, author: String) -> impl IntoView {
	let (count, set_count) = create_signal(n);
	let (clicked, set_clicked) = create_signal(true);
	let auth = use_context::<Auth>().expect("missing auth context");
	let privacy = use_context::<PrivacyControl>().expect("missing privacy context");
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
				let (mut to, cc) = privacy.get().address(&auth.username());
				to.push(author.clone());
				let payload = serde_json::Value::Object(serde_json::Map::default())
					.set_activity_type(Some(apb::ActivityType::Announce))
					.set_object(apb::Node::link(target.clone()))
					.set_to(apb::Node::links(to))
					.set_cc(apb::Node::links(cc));
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
