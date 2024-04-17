pub mod context;

use apb::{target::Addressed, Activity, Actor, Base, Collection, Object, ObjectMut};
use leptos::{leptos_dom::logging::{console_error, console_log}, *};
use leptos_router::*;

use crate::context::{Http, Timeline, Uri, CACHE};

pub const URL_BASE: &str = "https://feditest.alemi.dev";
pub const URL_PREFIX: &str = "/web";

#[derive(Debug, serde::Serialize)]
struct LoginForm {
	email: String,
	password: String,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Auth {
	pub token: String,
	pub user: String,
	pub expires: chrono::DateTime<chrono::Utc>,
}

pub trait MaybeToken {
	fn present(&self) -> bool;
	fn token(&self) -> String;
}

impl MaybeToken for Option<String> {
	fn token(&self) -> String {
		match self {
			None => String::new(),
			Some(x) => x.clone(),
		}
	}
	fn present(&self) -> bool {
		match self {
			None => false,
			Some(x) => !x.is_empty(),
		}
	}
}

#[component]
pub fn LoginBox(
	token_tx: WriteSignal<Option<String>>,
	token: Signal<Option<String>>,
	username: Signal<Option<String>>,
	username_tx: WriteSignal<Option<String>>,
	home_tl: Timeline,
) -> impl IntoView {
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class="w-100" class:hidden=move || !token.get().present() >
				"Hello "<a href={move || Uri::web("users", &username.get().unwrap_or_default() )} >{move || username.get().unwrap_or_default() }</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					token_tx.set(None);
				} />
			</div>
			<div class:hidden=move || token.get().present() >
				<input class="w-100" type="text" node_ref=username_ref placeholder="username" />
				<input class="w-100" type="text" node_ref=password_ref placeholder="password" />
				<input class="w-100" type="submit" value="login" on:click=move |_| {
					console_log("logging in...");
					let email = username_ref.get().map(|x| x.value()).unwrap_or("".into());
					let password = password_ref.get().map(|x| x.value()).unwrap_or("".into());
					spawn_local(async move {
						let auth = reqwest::Client::new()
							.post(format!("{URL_BASE}/auth"))
							.json(&LoginForm { email, password })
							.send()
							.await.unwrap()
							.json::<Auth>()
							.await.unwrap();
						console_log(&format!("logged in until {}", auth.expires));
						let username = auth.user.split('/').last().unwrap_or_default().to_string();
						// reset home feed and point it to our user's inbox
						home_tl.set_feed(vec![]);
						home_tl.set_next(format!("{URL_BASE}/users/{}/inbox/page", username));
						// update our username and token cookies
						username_tx.set(Some(username));
						token_tx.set(Some(auth.token));
					});
				} />
			</div>
		</div>
	}
}

#[component]
pub fn Navigator() -> impl IntoView {
	let auth = use_context::<Signal<Option<String>>>().expect("missing auth context");
	view! {	
		<a href="/web/home" >
			<input class="w-100"
				type="submit"
				class:hidden=move || !auth.get().present()
				// class:active=move || use_location().pathname.get().ends_with("/home")
				value="home timeline"
			/>
		</a>
		<a href="/web/server" >
			<input
				class="w-100"
				// class:active=move || use_location().pathname.get().ends_with("/server")
				type="submit"
				value="server timeline"
			/>
		</a>
		<a href="/web/about" >
			<input
				class="w-100"
				// class:active=move || use_location().pathname.get().ends_with("/server")
				type="submit"
				value="about"
			/>
		</a>
	}
}

#[component]
pub fn PostBox(username: Signal<Option<String>>) -> impl IntoView {
	let auth = use_context::<Signal<Option<String>>>().expect("missing auth context");
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let content_ref: NodeRef<html::Textarea> = create_node_ref();
	let public_ref: NodeRef<html::Input> = create_node_ref();
	let followers_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div class:hidden=move || !auth.get().present() >
			<table class="align w-100">
				<tr>
				<td><input type="checkbox" title="public" value="public" node_ref=public_ref /></td>
				<td class="w-100"><input class="w-100" type="text" node_ref=summary_ref title="summary" /></td>
				<td><input type="checkbox" title="followers" value="followers" node_ref=followers_ref checked /></td>
				</tr>
				<tr>
					<td colspan="3">
						<textarea rows="5" class="w-100" node_ref=content_ref placeholder="leptos is kinda fun!" ></textarea>
					</td>
				</tr>
				<tr>
					<td colspan="3">
						<button class="w-100" type="button" on:click=move |_| {
							spawn_local(async move {
								let summary = summary_ref.get().map(|x| x.value());
								let content = content_ref.get().map(|x| x.value()).unwrap_or_default();
								let public = public_ref.get().map(|x| x.checked()).unwrap_or_default();
								let followers = followers_ref.get().map(|x| x.checked()).unwrap_or_default();
								match Http::post(
									&format!("{URL_BASE}/users/test/outbox"),
									&serde_json::Value::Object(serde_json::Map::default())
										.set_object_type(Some(apb::ObjectType::Note))
										.set_summary(summary.as_deref())
										.set_content(Some(&content))
										.set_to(
											if public {
												apb::Node::links(vec![apb::target::PUBLIC.to_string()])
											} else { apb::Node::Empty }
										)
										.set_cc(
											if followers {
												apb::Node::links(vec![format!("{URL_BASE}/users/{}/followers", username.get().unwrap_or_default())])
											} else { apb::Node::Empty }
										),
									auth
								)
									.await
								{
									Err(e) => console_error(&format!("error posting note: {e}")),
									Ok(()) => {
										if let Some(x) = summary_ref.get() { x.set_value("") }
										if let Some(x) = content_ref.get() { x.set_value("") }
									},
								}
							})
						} >post</button>
					</td>
				</tr>
			</table>
		</div>
	}
}

#[component]
pub fn TimelinePicker(
	tx: WriteSignal<String>,
	rx: ReadSignal<String>,
) -> impl IntoView {
	let targets = (
		"https://feditest.alemi.dev/users/test/inbox/page".to_string(),
		"https://feditest.alemi.dev/users/test/outbox/page".to_string(),
		"https://feditest.alemi.dev/inbox/page".to_string(),
		"https://feditest.alemi.dev/outbox/page".to_string(),
	);
	let (my_in, my_out, our_in, our_out) = targets.clone();
	let (my_in_, my_out_, our_in_, our_out_) = targets;
	view! {
		<input type="submit" class:active=move || rx.get() == my_in_ on:click=move |_| tx.set(my_in.clone()) value="my inbox" />
		<input type="submit" class:active=move || rx.get() == my_out_ on:click=move |_| tx.set(my_out.clone()) value="my outbox" />
		<input type="submit" class:active=move || rx.get() == our_in_ on:click=move |_| tx.set(our_in.clone()) value="global inbox" />
		<input type="submit" class:active=move || rx.get() == our_out_ on:click=move |_| tx.set(our_out.clone()) value="global outbox" />
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
pub fn UserPage() -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Signal<Option<String>>>().expect("missing auth context");
	let actor = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), move |id| {
		async move {
			match CACHE.get(&Uri::full("users", &id)) {
				Some(x) => Some(x.clone()),
				None => {
					let user : serde_json::Value = Http::fetch(&Uri::api("users", &id), auth).await.ok()?;
					CACHE.put(Uri::full("users", &id), user.clone());
					Some(user)
				},
			}
		}
	});
	view! {
		<div>
			<div class="tl-header w-100 center mb-s" >view::user</div>
			<div>
				{move || match actor.get() {
					None => view! { <p>loading...</p> }.into_view(),
					Some(None) => view! { <p><code>error loading</code></p> }.into_view(),
					Some(Some(x)) => view! {
						<div class="ml-3 mr-3 mt-3">
							<ActorBanner object=x.clone() />
							<p 
								class="pb-2 pt-2 pr-2 pl-2"
								style={format!(
									"background-image: url({}); background-size: cover;",
									x.image().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default()
								)}
							>
								{
									dissolve::strip_html_tags(x.summary().unwrap_or(""))
										.into_iter()
										.map(|x| view! { <p>{x}</p> })
										.collect_view()
								}
							</p>
							<ul>
								<li><code>type</code>" "<b>{x.actor_type().unwrap_or(apb::ActorType::Person).as_ref().to_string()}</b></li>
								<li><code>following</code>" "<b>{x.following().get().map(|x| x.total_items().unwrap_or(0))}</b></li>
								<li><code>followers</code>" "<b>{x.followers().get().map(|x| x.total_items().unwrap_or(0))}</b></li>
								<li><code>created</code>" "{x.published().map(|x| x.to_rfc3339())}</li>
							</ul>
						</div>
						<hr />
						<TimelineFeed tl=Timeline::new(format!("{}/outbox/page", Uri::api("users", x.id().unwrap_or_default()))) />
					}.into_view(),
				}}
			</div>
		</div>
	}
}

#[component]
pub fn ObjectPage() -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Signal<Option<String>>>().expect("missing auth context");
	let object = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), move |oid| {
		async move {
			match CACHE.get(&Uri::full("objects", &oid)) {
				Some(x) => Some(x.clone()),
				None => {
					let obj = Http::fetch::<serde_json::Value>(&Uri::api("objects", &oid), auth).await.ok()?;
					CACHE.put(Uri::full("objects", &oid), obj.clone());
					Some(obj)
				}
			}
		}
	});
	view! {
		<div>
			<div class="tl-header w-100 center mb-s" >view::object</div>
			<div class="ma-2" >
				{move || match object.get() {
					Some(Some(o)) => view!{ <Object object=o /> }.into_view(),
					Some(None) => view! { <p><code>loading failed</code></p> }.into_view(),
					None => view! { <p> loading ... </p> }.into_view(),
				}}
			</div>
		</div>
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
pub fn About() -> impl IntoView {
	view! {
		<div>
			<div class="tl-header w-100 center mb-s" >about</div>
			<div class="mt-s mb-s" >
				<p><code>μpub</code>" is a micro social network powered by "<a href="">ActivityPub</a></p>
			</div>
		</div>
	}
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct OmgReqwestErrorIsNotClonable(String);

#[component]
pub fn TimelinePage(name: &'static str, tl: Timeline) -> impl IntoView {
	view! {
		<div>
			<div class="tl-header w-100 center mb-s" >{name}</div>
			<div class="mt-s mb-s" >
				<TimelineFeed tl=tl />
			</div>
		</div>
	}
}

#[component]
pub fn TimelineFeed(tl: Timeline) -> impl IntoView {
	let auth = use_context::<Signal<Option<String>>>().expect("missing auth context");
	view! {
		<For
			each=move || tl.feed.get()
			key=|k| k.to_string()
			children=move |id: String| {
				match CACHE.get(&id) {
					Some(object) => {
						view! {
							<div class="ml-1 mr-1 mt-1">
								<InlineActivity activity=object />
							</div>
							<hr/ >
						}.into_view()
					},
					None => view! {
						<p><code>{id}</code>" "[<a href={uri}>go</a>]</p>
					}.into_view(),
				}
			}
		/ >
		<div class="center" >
			<button type="button"
				on:click=move |_| {
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							console_error(&format!("error fetching more items for timeline: {e}"));
						}
					})
				}
			>more</button>
		</div>
	}
}
