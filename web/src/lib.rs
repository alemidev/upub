pub mod context;

use apb::{target::Addressed, Activity, ActivityMut, Actor, Base, Collection, Object, ObjectMut};
use context::CTX;
use leptos::{leptos_dom::logging::console_log, *};
use leptos_router::*;


pub const URL_BASE: &str = "https://feditest.alemi.dev";
pub const URL_PREFIX: &str = "/web";

#[derive(Debug, serde::Serialize)]
struct LoginForm {
	email: String,
	password: String,
}

fn web_uri(kind: &str, url: &str) -> String {
	if url.starts_with(URL_BASE) {
		format!("/web/{kind}/{}", url.split('/').last().unwrap_or_default())
	} else {
		format!("/web/{kind}/+{}", url.replace("https://", "").replace('/', "@"))
	}
}

fn api_uri(kind: &str, url: &str) -> String {
	if url.starts_with(URL_BASE) {
		url.to_string()
	} else {
		format!("{URL_BASE}/{kind}/+{}", url.replace("https://", "").replace('/', "@"))
	}
}

#[derive(Debug, serde::Deserialize)]
struct AuthSuccess {
	token: String,
	user: String,
	expires: chrono::DateTime<chrono::Utc>,
}

#[component]
pub fn LoginBox(
	rx: Signal<Option<String>>,
	tx: WriteSignal<Option<String>>,
) -> impl IntoView {
	let (username, username_set) = create_signal("".to_string());
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class="w-100" class:hidden=move || { rx.get().unwrap_or_default().is_empty() }>
				"Hello "<a href={move || web_uri("users", &username.get())} >{move || username.get()}</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					tx.set(None);
				} />
			</div>
			<div class:hidden=move || { !rx.get().unwrap_or_default().is_empty() }>
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
							.json::<AuthSuccess>()
							.await.unwrap();
						tx.set(Some(auth.token));
						username_set.set(auth.user);
						console_log(&format!("logged in until {}", auth.expires));
					});
				} />
			</div>
		</div>
	}
}

#[component]
pub fn PostBox(token: Signal<Option<String>>) -> impl IntoView {
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let content_ref: NodeRef<html::Textarea> = create_node_ref();
	view! {
		<div class:hidden=move || { token.get().unwrap_or_default().is_empty() }>
			<input class="w-100" type="text" node_ref=summary_ref placeholder="CW" />
			<textarea class="w-100" node_ref=content_ref placeholder="hello world!" ></textarea>
			<button class="w-100" type="button" on:click=move |_| {
				spawn_local(async move {
					let summary = summary_ref.get().map(|x| x.value());
					let content = content_ref.get().map(|x| x.value()).unwrap_or("".into());
					reqwest::Client::new()
						.post(format!("{URL_BASE}/users/test/outbox"))
						.header("Authorization", format!("Bearer {}", token.get().unwrap_or_default()))
						.json(
							&serde_json::Value::Object(serde_json::Map::default())
								.set_object_type(Some(apb::ObjectType::Note))
								.set_summary(summary.as_deref())
								.set_content(Some(&content))
								.set_to(apb::Node::links(vec![apb::target::PUBLIC.to_string()]))
								.set_cc(apb::Node::links(vec![format!("{URL_BASE}/users/test/followers")]))
						)
						.send()
						.await.unwrap()
						.error_for_status()
						.unwrap();
				})
			} >post</button>
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
			let uri = web_uri("users", &uid);
			let avatar_url = object.icon().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
			let display_name = object.name().unwrap_or_default().to_string();
			let username = object.preferred_username().unwrap_or_default().to_string();
			let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
			view! {
				<div>
					<table class="align" >
					<tr>
						<td rowspan="2" ><img class="avatar-circle" src={avatar_url} /></td>
						<td><b>{display_name}</b></td>
					</tr>
					<tr>
						<td class="top" ><a class="clean" href={uri} ><small>{username}@{domain}</small></a></td>
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
	let actor = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), |id| {
		async move {
			let uri = web_uri("users", &id);
			match CTX.cache.actors.get(&uri) {
				Some(x) => Some(x.clone()),
				None => {
					let user = reqwest::get(&uri)
						.await
						.ok()?
						.json::<serde_json::Value>()
						.await
						.ok()?;
					CTX.cache.actors.insert(uri, user.clone());
					Some(user)
				},
			}
		}
	});
	view! {
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
						{x.summary().unwrap_or("").to_string()}
					</p>
					<ul>
						<li><code>type</code>" "<b>{x.actor_type().unwrap_or(apb::ActorType::Person).as_ref().to_string()}</b></li>
						<li><code>following</code>" "<b>{x.following().get().map(|x| x.total_items().unwrap_or(0))}</b></li>
						<li><code>followers</code>" "<b>{x.followers().get().map(|x| x.total_items().unwrap_or(0))}</b></li>
						<li><code>created</code>" "{x.published().map(|x| x.to_rfc3339())}</li>
					</ul>
				</div>
			}.into_view(),
		}}
	}
}

#[component]
pub fn ObjectPage() -> impl IntoView {
	let params = use_params_map();
	let object = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), |oid| {
		async move {
			let uid = format!("{URL_BASE}/objects/{oid}");
			match CTX.cache.actors.get(&uid) {
				Some(x) => Some(x.clone()),
				None => reqwest::get(uid)
				.await
				.ok()?
				.json::<serde_json::Value>()
				.await
				.ok()
			}
		}
	});
	view! {
		{move || match object.get() {
			Some(Some(o)) => view!{ <Object object=o /> }.into_view(),
			Some(None) => view! { <p><code>loading failed</code></p> }.into_view(),
			None => view! { <p> loading ... </p> }.into_view(),
		}}
	}
}

#[component]
pub fn Object(object: serde_json::Value) -> impl IntoView {
	let summary = object.summary().unwrap_or_default().to_string();
	let content = object.content().unwrap_or_default().to_string();
	let date = object.published().map(|x| x.to_rfc3339()).unwrap_or_default();
	let author_id = object.attributed_to().id().unwrap_or_default();
	let author = CTX.cache.actors.get(&author_id).map(|x| view! { <ActorBanner object=x.clone() /> });
	view! {
		{author}
		<table>
			<tr>
				<td>{summary}</td>
			</tr>
			<tr>
				<td>{content}</td>
			</tr>
			<tr>
				<td>{date}</td>
			</tr>
		</table>
	}
}

#[component]
pub fn InlineActivity(activity: serde_json::Value) -> impl IntoView {
	let object = activity.clone().object().extract().unwrap_or_else(||
		serde_json::Value::String(activity.object().id().unwrap_or_default())
	);
	let object_id = object.id().unwrap_or_default().to_string();
	let object_uri = web_uri("objects", &object_id);
	let content = dissolve::strip_html_tags(object.content().unwrap_or_default());
	let addressed = activity.addressed();
	let audience = format!("[ {} ]", addressed.join(", "));
	let privacy = if addressed.iter().any(|x| x == apb::target::PUBLIC) {
		"[public]"
	} else if addressed.iter().any(|x| x.ends_with("/followers")) {
		"[followers]"
	} else {
		"[private]"
	};
	let title = object.summary().unwrap_or_default().to_string();
	let date = object.published().map(|x| x.to_rfc3339()).unwrap_or_else(||
		activity.published().map(|x| x.to_rfc3339()).unwrap_or_default()
	);
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		{match kind {
			// post
			apb::ActivityType::Create => view! {
				<div>
					<p><i>{title}</i></p>
					{
						content
							.into_iter()
							.map(|x| view! { <p>{x}</p> }.into_view())
							.collect::<Vec<View>>()
					}
				</div>
			},
			kind => view! {
				<div>
					<b>{kind.as_ref().to_string()}</b>" >> "<a href={object_uri}>{object_id}</a>
				</div>
			},
		}}
		<small><u title={audience} >{privacy}</u>" "{date}</small>
	}
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct OmgReqwestErrorIsNotClonable(String);

#[component]
pub fn Timeline(
	token: Signal<Option<String>>,
) -> impl IntoView {
	let (timeline, set_timeline) = create_signal(format!("{URL_BASE}/inbox/page"));
	let items = create_local_resource(move || timeline.get(), move |feed_url| async move {
		fetch_activities_with_users(&feed_url, token).await
	});
	view! {
		<div class="ml-1">
			<TimelinePicker tx=set_timeline rx=timeline />
			<div class="boxscroll" >
				<ErrorBoundary fallback=move |err| view! { <p>{format!("{:?}", err.get())}</p> } >
					{move || items.with(|x| match x {
						None => Ok(view! { <p>loading...</p> }.into_view()),
						Some(data) => match data {
							Err(e) => Err(OmgReqwestErrorIsNotClonable(e.to_string())),
							Ok(values) => Ok(
								values
									.iter()
									.map(|object| {
										let actor = object.actor().extract().unwrap_or_else(||
											 serde_json::Value::String(object.actor().id().unwrap_or_default())
										);
										view! {
											<div class="ml-1 mr-1 mt-1">
												<ActorBanner object=actor />
												<InlineActivity activity=object.clone() />
											</div>
											<hr/ >
										}
									})
									.collect::<Vec<Fragment>>()
									.into_view()
							),
						}
					})}
				</ErrorBoundary>
			</div>
		</div>
	}
}

async fn fetch_activities_with_users(
	feed_url: &str,
	token: Signal<Option<String>>,
) -> reqwest::Result<Vec<serde_json::Value>> {
	let mut req = reqwest::Client::new().get(feed_url);

	if let Some(token) = token.get() {
		req = req.header("Authorization", format!("Bearer {token}"));
	}

	let activities : Vec<serde_json::Value> = req
			.send()
			.await?
			.json::<serde_json::Value>()
			.await?
			.ordered_items()
			.collect();

	// i could make this fancier with iterators and futures::join_all but they would run
	// concurrently and make a ton of parallel request, we actually want these sequential because
	// first one may fetch same user as second one
	// some fancier logic may make a set of all actors and fetch uniques concurrently...
	let mut out = Vec::new();
	for x in activities {
		if let Some(uid) = x.actor().id() {
			if let Some(actor) = CTX.cache.actors.get(&uid) {
				out.push(x.set_actor(apb::Node::object(actor.clone())))
			} else {
				let mut req = reqwest::Client::new()
					.get(api_uri("users", &uid));

				if let Some(token) = token.get() {
					req = req.header("Authorization", format!("Bearer {token}"));
				}

				// TODO don't fail whole timeline fetch when one user fails fetching...
				let actor = req.send().await?.json::<serde_json::Value>().await?;
				CTX.cache.actors.insert(web_uri("users", &uid), actor.clone());

				out.push(x.set_actor(apb::Node::object(actor)))
			}
		} else {
			out.push(x)
		}
	}

	Ok(out)
}
