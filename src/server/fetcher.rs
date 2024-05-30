use std::collections::BTreeMap;

use apb::{target::Addressed, Activity, Actor, ActorMut, Base, Collection, Object};
use base64::Engine;
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{EntityTrait, IntoActiveModel, NotSet};

use crate::{errors::UpubError, model, VERSION};

use super::{addresser::Addresser, httpsign::HttpSignature, normalizer::Normalizer, Context};

#[derive(Debug, Clone)]
pub enum PullResult<T> {
	Actor(T),
	Activity(T),
	Object(T),
}

impl<T> PullResult<T> {
	pub fn inner(self) -> T {
		match self {
			Self::Actor(x) | Self::Activity(x) | Self::Object(x) => x
		}
	}
}

impl PullResult<serde_json::Value> {
	pub fn actor(self) -> crate::Result<serde_json::Value> {
		match self {
			Self::Actor(x) => Ok(x),
			Self::Activity(x) => Err(UpubError::Mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Err(UpubError::Mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn activity(self) -> crate::Result<serde_json::Value> {
		match self {
			Self::Actor(x) => Err(UpubError::Mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Ok(x),
			Self::Object(x) => Err(UpubError::Mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn object(self) -> crate::Result<serde_json::Value> {
		match self {
			Self::Actor(x) => Err(UpubError::Mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Err(UpubError::Mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Ok(x),
		}
	}

	pub async fn resolve(self, ctx: &(impl Fetcher + Sync)) -> crate::Result<()> {
		match self {
			Self::Actor(x) => { ctx.resolve_user(x).await?; },
			Self::Object(x) => { ctx.resolve_object(x).await?; },
			Self::Activity(x) => { ctx.resolve_activity(x).await?; },
		}
		Ok(())
	}
}

#[axum::async_trait]
pub trait Fetcher {
	async fn pull(&self, id: &str) -> crate::Result<PullResult<serde_json::Value>> { self.pull_r(id, 0).await }
	async fn pull_r(&self, id: &str, depth: i32) -> crate::Result<PullResult<serde_json::Value>>;


	async fn webfinger(&self, user: &str, host: &str) -> crate::Result<String>;

	async fn fetch_domain(&self, domain: &str) -> crate::Result<model::instance::Model>;

	async fn fetch_user(&self, id: &str) -> crate::Result<model::actor::Model>;
	async fn resolve_user(&self, actor: serde_json::Value) -> crate::Result<model::actor::Model>;

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model>;
	async fn resolve_activity(&self, activity: serde_json::Value) -> crate::Result<model::activity::Model>;

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model> { self.fetch_object_r(id, 0).await }
	#[allow(unused)] async fn resolve_object(&self, object: serde_json::Value) -> crate::Result<model::object::Model> { self.resolve_object_r(object, 0).await }

	async fn fetch_object_r(&self, id: &str, depth: u32) -> crate::Result<model::object::Model>;
	async fn resolve_object_r(&self, object: serde_json::Value, depth: u32) -> crate::Result<model::object::Model>;


	async fn fetch_thread(&self, id: &str) -> crate::Result<()>;

	async fn request(
		method: reqwest::Method,
		url: &str,
		payload: Option<&str>,
		from: &str,
		key: &str,
		domain: &str,
	) -> crate::Result<Response> {
		let host = Context::server(url);
		let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(); // lmao @ "GMT"
		let path = url.replace("https://", "").replace("http://", "").replace(&host, "");
		let digest = format!("sha-256={}",
			base64::prelude::BASE64_STANDARD.encode(
				openssl::sha::sha256(payload.unwrap_or("").as_bytes())
			)
		);

		let headers = vec!["(request-target)", "host", "date", "digest"];
		let headers_map : BTreeMap<String, String> = [
			("host".to_string(), host.clone()),
			("date".to_string(), date.clone()),
			("digest".to_string(), digest.clone()),
		].into();

		let mut signer = HttpSignature::new(
			format!("{from}#main-key"), // TODO don't hardcode #main-key
			//"hs2019".to_string(), // pixelfeed/iceshrimp made me go back
			"rsa-sha256".to_string(),
			&headers,
		);
		
		signer
			.build_manually(&method.to_string().to_lowercase(), &path, headers_map)
			.sign(key)?;

		let response = reqwest::Client::new()
			.request(method.clone(), url)
			.header(ACCEPT, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(USER_AGENT, format!("upub+{VERSION} ({domain})"))
			.header("Host", host.clone())
			.header("Date", date.clone())
			.header("Digest", digest)
			.header("Signature", signer.header())
			.body(payload.unwrap_or("").to_string())
			.send()
			.await?;

		// TODO this is ugly but i want to see the raw response text when it's a failure
		match response.error_for_status_ref() {
			Ok(_) => Ok(response),
			Err(e) => Err(UpubError::FetchError(e, response.text().await?)),
		}
	}
}


#[axum::async_trait]
impl Fetcher for Context {
	async fn pull_r(&self, id: &str, depth: u32) -> crate::Result<PullResult<serde_json::Value>> {
		let _domain = self.fetch_domain(&Context::server(id)).await?;

		let document = Self::request(
			Method::GET, id, None,
			&format!("https://{}", self.domain()), self.pkey(), self.domain(),
		)
			.await?
			.json::<serde_json::Value>()
			.await?;

		let doc_id = document.id().ok_or_else(|| UpubError::field("id"))?;
		if id != doc_id {
			if depth >= self.cfg().security.max_id_redirects {
				return Err(UpubError::unprocessable());
			}
			return self.pull(doc_id).await;
		}

		match document.object_type() {
			None => Err(UpubError::bad_request()),
			Some(apb::ObjectType::Collection(_)) => Err(UpubError::unprocessable()),
			Some(apb::ObjectType::Tombstone) => Err(UpubError::not_found()),
			Some(apb::ObjectType::Activity(_)) => Ok(PullResult::Activity(document)),
			Some(apb::ObjectType::Actor(_)) => Ok(PullResult::Actor(document)),
			_ => Ok(PullResult::Object(document)),
		}
	}


	async fn webfinger(&self, user: &str, host: &str) -> crate::Result<String> {
		let subject = format!("acct:{user}@{host}");
		let webfinger_uri = format!("https://{host}/.well-known/webfinger?resource={subject}");
		let resource = reqwest::Client::new()
			.get(webfinger_uri)
			.header(ACCEPT, "application/jrd+json")
			.header(USER_AGENT, format!("upub+{VERSION} ({})", self.domain()))
			.send()
			.await?
			.json::<jrd::JsonResourceDescriptor>()
			.await?;

		if resource.subject != subject {
			return Err(UpubError::unprocessable());
		}

		for link in resource.links {
			if link.rel == "self" {
				if let Some(href) = link.href {
					return Ok(href);
				}
			}
		}

		if let Some(alias) = resource.aliases.into_iter().next() {
			return Ok(alias);
		}

		Err(UpubError::not_found())
	}

	async fn fetch_domain(&self, domain: &str) -> crate::Result<model::instance::Model> {
		if let Some(x) = model::instance::Entity::find_by_domain(domain).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let mut instance_model = model::instance::Model {
			internal: 0,
			domain: domain.to_string(),
			name: None,
			software: None,
			down_since: None,
			icon: None,
			version: None,
			users: None,
			posts: None,
			published: chrono::Utc::now(),
			updated: chrono::Utc::now(),
		};

		if let Ok(res) = Self::request(
			Method::GET, &format!("https://{domain}"), None, &format!("https://{}", self.domain()), self.pkey(), self.domain(),
		).await {
			if let Ok(actor) = res.json::<serde_json::Value>().await {
				if let Some(name) = actor.name() {
					instance_model.name = Some(name.to_string());
				}
				if let Some(icon) = actor.icon().id() {
					instance_model.icon = Some(icon);
				}
			}
		}

		if let Ok(nodeinfo) = model::instance::Entity::nodeinfo(domain).await {
			instance_model.software = Some(nodeinfo.software.name);
			instance_model.version = nodeinfo.software.version;
			instance_model.users = nodeinfo.usage.users.and_then(|x| x.total);
			instance_model.posts = nodeinfo.usage.local_posts;
		}

		let mut active_model = instance_model.clone().into_active_model();
		active_model.internal = NotSet;
		model::instance::Entity::insert(active_model).exec(self.db()).await?;

		let internal = model::instance::Entity::domain_to_internal(domain, self.db()).await?;
		instance_model.internal = internal;

		Ok(instance_model)
	}

	async fn resolve_user(&self, mut document: serde_json::Value) -> crate::Result<model::actor::Model> {
		let id = document.id().ok_or_else(|| UpubError::field("id"))?.to_string();

		// TODO try fetching these numbers from audience/generator fields to avoid making 2 more GETs every time
		if let Some(followers_url) = &document.followers().id() {
			let req = Self::request(
				Method::GET, followers_url, None,
				&format!("https://{}", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_followers) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_followers.total_items() {
						document = document.set_followers_count(Some(total));
					}
				}
			}
		}

		if let Some(following_url) = &document.following().id() {
			let req =  Self::request(
				Method::GET, following_url, None,
				&format!("https://{}", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_following) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_following.total_items() {
						document = document.set_following_count(Some(total));
					}
				}
			}
		}

		let user_model = model::actor::ActiveModel::new(&document)?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		model::actor::Entity::insert(user_model).exec(self.db()).await?;
		
		// TODO fetch it back to get the internal id
		Ok(
			model::actor::Entity::find_by_ap_id(&id)
				.one(self.db())
				.await?
				.ok_or_else(UpubError::internal_server_error)?
		)
	}

	async fn fetch_user(&self, id: &str) -> crate::Result<model::actor::Model> {
		if let Some(x) = model::actor::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let document = self.pull(id).await?.actor()?;

		self.resolve_user(document).await
	}

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let activity = self.pull(id).await?.activity()?;

		self.resolve_activity(activity).await
	}

	async fn resolve_activity(&self, activity: serde_json::Value) -> crate::Result<model::activity::Model> {
		let id = activity.id().ok_or_else(|| UpubError::field("id"))?.to_string();

		if let Some(activity_actor) = activity.actor().id() {
			if let Err(e) = self.fetch_user(&activity_actor).await {
				tracing::warn!("could not get actor of fetched activity: {e}");
			}
		}

		if let Some(activity_object) = activity.object().id() {
			if let Err(e) = self.fetch_object(&activity_object).await {
				tracing::warn!("could not get object of fetched activity: {e}");
			}
		}

		let activity_model = self.insert_activity(activity, Some(Context::server(&id))).await?;

		let addressed = activity_model.addressed();
		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(Some(activity_model.internal), None, &expanded_addresses).await?;

		Ok(activity_model)
	}

	async fn fetch_thread(&self, _id: &str) -> crate::Result<()> {
		// crawl_replies(self, id, 0).await
		todo!()
	}

	async fn fetch_object_r(&self, id: &str, depth: u32) -> crate::Result<model::object::Model> {
		if let Some(x) = model::object::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let object = self.pull(id).await?.object()?;

		self.resolve_object_r(object, depth).await
	}

	async fn resolve_object_r(&self, object: serde_json::Value, depth: u32) -> crate::Result<model::object::Model> {
		let id = object.id().ok_or_else(|| UpubError::field("id"))?.to_string();

		if let Some(oid) = object.id() {
			if oid != id {
				if let Some(x) = model::object::Entity::find_by_ap_id(oid).one(self.db()).await? {
					return Ok(x); // already in db, but with id different that given url
				}
			}
		}

		if let Some(attributed_to) = object.attributed_to().id() {
			if let Err(e) = self.fetch_user(&attributed_to).await {
				tracing::warn!("could not get actor of fetched object: {e}");
			}
		}

		let addressed = object.addressed();

		if let Some(reply) = object.in_reply_to().id() {
			if depth <= self.cfg().security.thread_crawl_depth {
				self.fetch_object_r(&reply, depth + 1).await?;
			} else {
				tracing::warn!("thread deeper than {}, giving up fetching more replies", self.cfg().security.thread_crawl_depth);
			}
		}

		let object_model = self.insert_object(object, None).await?;

		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(None, Some(object_model.internal), &expanded_addresses).await?;

		Ok(object_model)
	}
}

#[axum::async_trait]
pub trait Fetchable : Sync + Send {
	async fn fetch(&mut self, ctx: &crate::server::Context) -> crate::Result<&mut Self>;
}

#[axum::async_trait]
impl Fetchable for apb::Node<serde_json::Value> {
	async fn fetch(&mut self, ctx: &crate::server::Context) -> crate::Result<&mut Self> {
		if let apb::Node::Link(uri) = self {
			*self = Context::request(Method::GET, uri.href(), None, ctx.base(), ctx.pkey(), ctx.domain())
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}

		Ok(self)
	}
}

// #[async_recursion::async_recursion]
// async fn crawl_replies(ctx: &Context, id: &str, depth: usize) -> crate::Result<()> {
// 	tracing::info!("crawling replies of '{id}'");
// 	let object = Context::request(
// 		Method::GET, id, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
// 	).await?.json::<serde_json::Value>().await?;
// 
// 	let object_model = model::object::Model::new(&object)?;
// 	match model::object::Entity::insert(object_model.into_active_model())
// 		.exec(ctx.db()).await
// 	{
// 		Ok(_) => {},
// 		Err(sea_orm::DbErr::RecordNotInserted) => {},
// 		Err(sea_orm::DbErr::Exec(_)) => {}, // ughhh bad fix for sqlite
// 		Err(e) => return Err(e.into()),
// 	}
// 
// 	if depth > 16 {
// 		tracing::warn!("stopping thread crawling: too deep!");
// 		return Ok(());
// 	}
// 
// 	let mut page_url = match object.replies().get() {
// 		Some(serde_json::Value::String(x)) => {
// 			let replies = Context::request(
// 				Method::GET, x, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
// 			).await?.json::<serde_json::Value>().await?;
// 			replies.first().id()
// 		},
// 		Some(serde_json::Value::Object(x)) => {
// 			let obj = serde_json::Value::Object(x.clone()); // lol putting it back, TODO!
// 			obj.first().id()
// 		},
// 		_ => return Ok(()),
// 	};
// 
// 	while let Some(ref url) = page_url {
// 		let replies = Context::request(
// 			Method::GET, url, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
// 		).await?.json::<serde_json::Value>().await?;
// 
// 		for reply in replies.items() {
// 			// TODO right now it crawls one by one, could be made in parallel but would be quite more
// 			// abusive, so i'll keep it like this while i try it out
// 			crawl_replies(ctx, reply.href(), depth + 1).await?;
// 		}
// 
// 		page_url = replies.next().id();
// 	}
// 
// 	Ok(())
// }
