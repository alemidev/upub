use apb::{Actor, Base, Collection, Object};

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>about</Breadcrumb>
			<div class="mt-s mb-s" >
				<p><code>μpub</code>" is a micro social network powered by "<a href="">ActivityPub</a></p>
				<p><i>"the "<a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>" is an ensemble of social networks, which, while independently hosted, can communicate with each other"</i></p>
				<p>content is aggregated in timelines, logged out users can only access global server timeline</p>
			</div>
		</div>
	}
}

#[component]
pub fn ConfigPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>config</Breadcrumb>
			<div class="mt-s mb-s" >
				<p><code>"not implemented :("</code></p>
			</div>
		</div>
	}
}

#[component]
pub fn UserPage() -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
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
			<Breadcrumb back=true >users::view</Breadcrumb>
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
	let auth = use_context::<Auth>().expect("missing auth context");
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
			<Breadcrumb back=true >objects::view</Breadcrumb>
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
pub fn TimelinePage(name: &'static str, tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<div>
			<Breadcrumb back=false>
				{name}
				<a class="clean ml-1" href="#" on:click=move |_| {
					tl.reset(tl.next.get().split('?').next().unwrap_or_default().to_string());
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							tracing::error!("error fetching more items for timeline: {e}");
						}
					})
				}><span class="emoji">
					"\u{1f5d8}"
				</span></a>
			</Breadcrumb>
			<div class="mt-s mb-s" >
				<TimelineFeed tl=tl />
			</div>
		</div>
	}
}
