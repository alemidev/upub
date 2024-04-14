use std::sync::Arc;

use apb::{target::Addressed, Activity, ActivityMut, Actor, Base, Collection, Object, ObjectMut};
use dashmap::DashMap;
use leptos::{leptos_dom::logging::console_log, *};

pub const BASE_URL: &str = "https://feditest.alemi.dev";

#[derive(Debug, serde::Serialize)]
struct LoginForm {
	email: String,
	password: String,
}

#[component]
pub fn LoginBox(
	rx: Signal<Option<String>>,
	tx: WriteSignal<Option<String>>,
) -> impl IntoView {
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class:hidden=move || { rx.get().unwrap_or_default().is_empty() }>
				<input type="submit" value="logout" on:click=move |_| {
					tx.set(None);
				} />
			</div>
			<div class:hidden=move || { !rx.get().unwrap_or_default().is_empty() }>
				<p>
					<input type="text" node_ref=username_ref placeholder="username" />
					<input type="text" node_ref=password_ref placeholder="password" />
					<input type="submit" value="login" on:click=move |_| {
						console_log("logging in...");
						let email = username_ref.get().map(|x| x.value()).unwrap_or("".into());
						let password = password_ref.get().map(|x| x.value()).unwrap_or("".into());
						spawn_local(async move {
							let auth = reqwest::Client::new()
								.post(format!("{BASE_URL}/auth"))
								.json(&LoginForm { email, password })
								.send()
								.await.unwrap()
								.json::<String>()
								.await.unwrap();
							tx.set(Some(auth));
						});
					} />
				</p>
			</div>
		</div>
	}
}

#[component]
pub fn PostBox(token: Signal<Option<String>>) -> impl IntoView {
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let content_ref: NodeRef<html::Textarea> = create_node_ref();
	view! {
		<div>
			<input class="w-100" type="text" node_ref=summary_ref placeholder="CW" />
			<textarea class="w-100" node_ref=content_ref placeholder="hello world!" ></textarea>
			<button class="w-100" type="button" on:click=move |_| {
				spawn_local(async move {
					let summary = summary_ref.get().map(|x| x.value());
					let content = content_ref.get().map(|x| x.value()).unwrap_or("".into());
					reqwest::Client::new()
						.post(format!("{BASE_URL}/users/test/outbox"))
						.header("Authorization", format!("Bearer {}", token.get().unwrap_or_default()))
						.json(
							&serde_json::Value::Object(serde_json::Map::default())
								.set_object_type(Some(apb::ObjectType::Note))
								.set_summary(summary.as_deref())
								.set_content(Some(&content))
								.set_to(apb::Node::links(vec![apb::target::PUBLIC.to_string()]))
								.set_cc(apb::Node::links(vec![format!("{BASE_URL}/users/test/followers")]))
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
		<p>
			<input type="submit" class:active=move || rx.get() == my_in_ on:click=move |_| tx.set(my_in.clone()) value="my inbox" />
			<input type="submit" class:active=move || rx.get() == my_out_ on:click=move |_| tx.set(my_out.clone()) value="my outbox" />
			<input type="submit" class:active=move || rx.get() == our_in_ on:click=move |_| tx.set(our_in.clone()) value="global inbox" />
			<input type="submit" class:active=move || rx.get() == our_out_ on:click=move |_| tx.set(our_out.clone()) value="global outbox" />
		</p>
	}
}

#[component]
pub fn Actor(object: serde_json::Value) -> impl IntoView {
	match object {
		serde_json::Value::String(id) => view! {
			<div><b>{id}</b></div>
		},
		serde_json::Value::Object(_) => {
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
						<td class="top" ><small>{username}@{domain}</small></td>
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
pub fn Activity(activity: serde_json::Value) -> impl IntoView {
	let object = activity.clone().object().extract().unwrap_or_else(||
		serde_json::Value::String(activity.object().id().unwrap_or_default())
	);
	let object_id = object.id().unwrap_or_default().to_string();
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
	let date = object.published().map(|x| x.to_rfc3339()).unwrap_or_default();
	let kind = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
	view! {
		{match kind {
			// post
			apb::ActivityType::Create => view! {
				<div>
					<p><i>{title}</i></p>
					<For
						each=move || content.clone() // TODO wtf this clone??
						key=|x| x.to_string() // TODO what about this clone?
						children=move |x: String| view! { <p>{x}</p> }
					/>
				</div>
			},
			kind => view! {
				<div>
					<b>{kind.as_ref().to_string()}</b>" >> "<i>{object_id}</i>
				</div>
			},
		}}
		<small><u title={audience} >{privacy}</u>" "{date}</small>
	}
}

#[component]
pub fn Timeline(
	token: Signal<Option<String>>,
) -> impl IntoView {
	let (timeline, set_timeline) = create_signal(format!("{BASE_URL}/inbox/page"));
	let users : Arc<DashMap<String, serde_json::Value>> = Arc::new(DashMap::new());
	let _users = users.clone(); // TODO i think there is syntactic sugar i forgot?
	let items = create_resource(move || timeline.get(), move |feed_url| {
		let __users = _users.clone(); // TODO lmao this is meme tier
		async move {
			let mut req = reqwest::Client::new().get(feed_url);

			if let Some(token) = token.get() {
				req = req.header("Authorization", format!("Bearer {token}"));
			}

			let activities : Vec<serde_json::Value> = req
					.send()
					.await.unwrap()
					.json::<serde_json::Value>()
					.await.unwrap()
					.ordered_items()
					.collect();

			// i could make this fancier with iterators and futures::join_all but they would run
			// concurrently and make a ton of parallel request, we actually want these sequential because
			// first one may fetch same user as second one
			// some fancier logic may make a set of all actors and fetch uniques concurrently...
			let mut out = Vec::new();
			for x in activities {
				if let Some(uid) = x.actor().id() {
					if let Some(actor) = __users.get(&uid) {
						out.push(x.set_actor(apb::Node::object(actor.clone())))
					} else {
						let mut req = reqwest::Client::new()
							.get(format!("https://feditest.alemi.dev/users/+?id={uid}"));

						if let Some(token) = token.get() {
							req = req.header("Authorization", format!("Bearer {token}"));
						}

						let actor = req.send().await.unwrap().json::<serde_json::Value>().await.unwrap();
						__users.insert(uid, actor.clone());

						out.push(x.set_actor(apb::Node::object(actor)))
					}
				} else {
					out.push(x)
				}
			}

			out
		}
	});
	view! {
		<div class="ml-1">
			<TimelinePicker tx=set_timeline rx=timeline />
			{move || match items.get() {
				None => view! { <p>loading...</p> }.into_view(),
				Some(data) => {
					view! {
						<For
							each=move || data.clone() // TODO wtf this clone??
							key=|x| x.id().unwrap_or("").to_string()
							children=move |x: serde_json::Value| {
								let actor = x.actor().extract().unwrap_or_else(||
									 serde_json::Value::String(x.actor().id().unwrap_or_default())
								);
								view! {
									<div class="post-card ml-1 mr-1">
										<Actor object=actor />
										<Activity activity=x />
									</div>
									<hr/ >
								}
							}
						/>
					}.into_view()
				},
			}}
		</div>
	}
}
