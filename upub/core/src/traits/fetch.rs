use std::collections::BTreeMap;

use apb::{target::Addressed, Activity, Actor, ActorMut, Base, Collection, Object};
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{DbErr, EntityTrait, IntoActiveModel, NotSet, TransactionTrait};

use crate::traits::normalize::AP;

use super::{Addresser, Normalizer};
use httpsign::HttpSignature;

#[derive(Debug, Clone)]
pub enum Pull<T> {
	Actor(T),
	Activity(T),
	Object(T),
}

#[derive(Debug, thiserror::Error)]
pub enum PullError {
	#[error("dereferenced resource ({0:?}) doesn't match requested type ({1:?})")]
	Mismatch(apb::ObjectType, apb::ObjectType),

	#[error("error fetching resource: {0:?}")]
	Reqwest(#[from] reqwest::Error),

	#[error("fetch failed with status {0}: {1}")]
	Fetch(reqwest::StatusCode, String),

	#[error("database error while fetching resource: {0:?}")]
	Database(#[from] sea_orm::DbErr),

	#[error("dereferenced resource is malformed: {0:?}")]
	Malformed(#[from] apb::FieldErr),

	#[error("error normalizing resource: {0:?}")]
	Normalization(#[from] crate::traits::normalize::NormalizerError),

	#[error("too many redirects while resolving resource id, aborting")]
	TooManyRedirects,

	#[error("resource no longer exists")]
	Tombstone,

	#[error("error constructing http signature: {0:?}")]
	HttpSignature(#[from] httpsign::HttpSignatureError),
}

impl PullError {
	fn mismatch(expected: apb::ObjectType, found: apb::ObjectType) -> Self {
		PullError::Mismatch(expected, found)
	}
}

impl Pull<serde_json::Value> {
	pub fn actor(self) -> Result<serde_json::Value, PullError> {
		match self {
			Self::Actor(x) => Ok(x),
			Self::Activity(x) => Err(PullError::mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Err(PullError::mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn activity(self) -> Result<serde_json::Value, PullError> {
		match self {
			Self::Actor(x) => Err(PullError::mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Ok(x),
			Self::Object(x) => Err(PullError::mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn object(self) -> Result<serde_json::Value, PullError> {
		match self {
			Self::Actor(x) => Err(PullError::mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Err(PullError::mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Ok(x),
		}
	}
}

#[async_trait::async_trait]
pub trait Fetcher {
	async fn pull(&self, id: &str) -> Result<Pull<serde_json::Value>, PullError> { self.pull_r(id, 0).await }
	async fn pull_r(&self, id: &str, depth: u32) -> Result<Pull<serde_json::Value>, PullError>;


	async fn webfinger(&self, user: &str, host: &str) -> Result<Option<String>, PullError>;

	async fn fetch_domain(&self, domain: &str) -> Result<crate::model::instance::Model, PullError>;

	async fn fetch_user(&self, id: &str) -> Result<crate::model::actor::Model, PullError>;
	async fn resolve_user(&self, actor: serde_json::Value) -> Result<crate::model::actor::Model, PullError>;

	async fn fetch_activity(&self, id: &str) -> Result<crate::model::activity::Model, PullError>;
	async fn resolve_activity(&self, activity: serde_json::Value) -> Result<crate::model::activity::Model, PullError>;

	async fn fetch_object(&self, id: &str) -> Result<crate::model::object::Model, PullError> { self.fetch_object_r(id, 0).await }
	async fn resolve_object(&self, object: serde_json::Value) -> Result<crate::model::object::Model, PullError> { self.resolve_object_r(object, 0).await }

	async fn fetch_object_r(&self, id: &str, depth: u32) -> Result<crate::model::object::Model, PullError>;
	async fn resolve_object_r(&self, object: serde_json::Value, depth: u32) -> Result<crate::model::object::Model, PullError>;


	async fn fetch_thread(&self, id: &str) -> Result<(), PullError>;

	async fn request(
		method: reqwest::Method,
		url: &str,
		payload: Option<&str>,
		from: &str,
		key: &str,
		domain: &str,
	) -> Result<Response, PullError> {
		let host = crate::Context::server(url);
		let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(); // lmao @ "GMT"
		let path = url.replace("https://", "").replace("http://", "").replace(&host, "");
		let digest = httpsign::digest(payload.unwrap_or_default());

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
			.header(USER_AGENT, format!("upub+{} ({domain})", crate::VERSION))
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
			Err(e) =>
				Err(PullError::Fetch(
					e.status().unwrap_or_default(),
					response.text().await?,
				)),
		}
	}
}


#[async_trait::async_trait]
impl Fetcher for crate::Context {
	async fn pull_r(&self, id: &str, depth: u32) -> Result<Pull<serde_json::Value>, PullError> {
		let _domain = self.fetch_domain(&crate::Context::server(id)).await?;

		let document = Self::request(
			Method::GET, id, None,
			&format!("https://{}/", self.domain()), self.pkey(), self.domain(),
		)
			.await?
			.json::<serde_json::Value>()
			.await?;

		let doc_id = document.id()?;
		if id != doc_id {
			if depth >= self.cfg().security.max_id_redirects {
				return Err(PullError::TooManyRedirects);
			}
			return self.pull(doc_id).await;
		}

		match document.object_type()? {
			apb::ObjectType::Collection(x) => Err(PullError::mismatch(apb::ObjectType::Object, apb::ObjectType::Collection(x))),
			apb::ObjectType::Tombstone => Err(PullError::Tombstone),
			apb::ObjectType::Activity(_) => Ok(Pull::Activity(document)),
			apb::ObjectType::Actor(_) => Ok(Pull::Actor(document)),
			_ => Ok(Pull::Object(document)),
		}
	}


	async fn webfinger(&self, user: &str, host: &str) -> Result<Option<String>, PullError> {
		let subject = format!("acct:{user}@{host}");
		let webfinger_uri = format!("https://{host}/.well-known/webfinger?resource={subject}");
		let resource = reqwest::Client::new()
			.get(webfinger_uri)
			.header(ACCEPT, "application/jrd+json")
			.header(USER_AGENT, format!("upub+{} ({})", crate::VERSION, self.domain()))
			.send()
			.await?
			.json::<jrd::JsonResourceDescriptor>()
			.await?;

		if resource.subject != subject {
			tracing::error!("webfinger result ({}) differs from expected subject ({})", resource.subject, subject);
			return Ok(None);
		}

		for link in resource.links {
			if link.rel == "self" {
				if let Some(href) = link.href {
					return Ok(Some(href));
				}
			}
		}

		if let Some(alias) = resource.aliases.into_iter().next() {
			return Ok(Some(alias));
		}

		Ok(None)
	}

	async fn fetch_domain(&self, domain: &str) -> Result<crate::model::instance::Model, PullError> {
		if let Some(x) = crate::model::instance::Entity::find_by_domain(domain).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let mut instance_model = crate::model::instance::Model {
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
			Method::GET, &format!("https://{domain}"), None,
			&format!("https://{}/", self.domain()), self.pkey(), self.domain(),
		).await {
			if let Ok(actor) = res.json::<serde_json::Value>().await {
				if let Ok(name) = actor.name() {
					instance_model.name = Some(name.to_string());
				}
				if let Ok(icon) = actor.icon().id() {
					instance_model.icon = Some(icon.to_string());
				}
			}
		}

		if let Ok(nodeinfo) = crate::model::instance::Entity::nodeinfo(domain).await {
			instance_model.software = Some(nodeinfo.software.name);
			instance_model.version = nodeinfo.software.version;
			instance_model.users = nodeinfo.usage.users.and_then(|x| x.total);
			instance_model.posts = nodeinfo.usage.local_posts;
		}

		let mut active_model = instance_model.clone().into_active_model();
		active_model.internal = NotSet;
		crate::model::instance::Entity::insert(active_model).exec(self.db()).await?;
		let internal = crate::model::instance::Entity::domain_to_internal(domain, self.db())
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(domain.to_string()))?;
		instance_model.internal = internal;

		Ok(instance_model)
	}

	async fn resolve_user(&self, mut document: serde_json::Value) -> Result<crate::model::actor::Model, PullError> {
		let id = document.id()?.to_string();

		// TODO try fetching these numbers from audience/generator fields to avoid making 2 more GETs every time
		if let Ok(followers_url) = document.followers().id() {
			let req = Self::request(
				Method::GET, followers_url, None,
				&format!("https://{}/", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_followers) = res.json::<serde_json::Value>().await {
					if let Ok(total) = user_followers.total_items() {
						document = document.set_followers_count(Some(total));
					}
				}
			}
		}

		if let Ok(following_url) = document.following().id() {
			let req = Self::request(
				Method::GET, following_url, None,
				&format!("https://{}/", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_following) = res.json::<serde_json::Value>().await {
					if let Ok(total) = user_following.total_items() {
						document = document.set_following_count(Some(total));
					}
				}
			}
		}

		let user_model = AP::actor_q(&document)?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		crate::model::actor::Entity::insert(user_model).exec(self.db()).await?;
		
		// TODO fetch it back to get the internal id
		Ok(
			crate::model::actor::Entity::find_by_ap_id(&id)
				.one(self.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(id.to_string()))?
		)
	}

	async fn fetch_user(&self, id: &str) -> Result<crate::model::actor::Model, PullError> {
		if let Some(x) = crate::model::actor::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let document = self.pull(id).await?.actor()?;

		self.resolve_user(document).await
	}

	async fn fetch_activity(&self, id: &str) -> Result<crate::model::activity::Model, PullError> {
		if let Some(x) = crate::model::activity::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let activity = self.pull(id).await?.activity()?;

		self.resolve_activity(activity).await
	}

	async fn resolve_activity(&self, activity: serde_json::Value) -> Result<crate::model::activity::Model, PullError> {
		if let Ok(activity_actor) = activity.actor().id() {
			if let Err(e) = self.fetch_user(activity_actor).await {
				tracing::warn!("could not get actor of fetched activity: {e}");
			}
		}

		if let Ok(activity_object) = activity.object().id() {
			if let Err(e) = self.fetch_object(activity_object).await {
				tracing::warn!("could not get object of fetched activity: {e}");
			}
		}

		let tx = self.db().begin().await?;

		let activity_model = self.insert_activity(activity, &tx).await?;

		let addressed = activity_model.addressed();
		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(Some(activity_model.internal), None, &expanded_addresses, &tx).await?;

		tx.commit().await?;

		Ok(activity_model)
	}

	async fn fetch_thread(&self, _id: &str) -> Result<(), PullError> {
		// crawl_replies(self, id, 0).await
		todo!()
	}

	async fn fetch_object_r(&self, id: &str, depth: u32) -> Result<crate::model::object::Model, PullError> {
		if let Some(x) = crate::model::object::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let object = self.pull(id).await?.object()?;

		self.resolve_object_r(object, depth).await
	}

	async fn resolve_object_r(&self, object: serde_json::Value, depth: u32) -> Result<crate::model::object::Model, PullError> {
		let id = object.id()?.to_string();

		if let Ok(oid) = object.id() {
			if oid != id {
				if let Some(x) = crate::model::object::Entity::find_by_ap_id(oid).one(self.db()).await? {
					return Ok(x); // already in db, but with id different that given url
				}
			}
		}

		if let Ok(attributed_to) = object.attributed_to().id() {
			if let Err(e) = self.fetch_user(attributed_to).await {
				tracing::warn!("could not get actor of fetched object: {e}");
			}
		}

		let addressed = object.addressed();

		if let Ok(reply) = object.in_reply_to().id() {
			if depth <= self.cfg().security.thread_crawl_depth {
				self.fetch_object_r(reply, depth + 1).await?;
			} else {
				tracing::warn!("thread deeper than {}, giving up fetching more replies", self.cfg().security.thread_crawl_depth);
			}
		}

		let tx = self.db().begin().await?;

		let object_model = self.insert_object(object, &tx).await?;

		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(None, Some(object_model.internal), &expanded_addresses, &tx).await?;

		tx.commit().await?;

		Ok(object_model)
	}
}

#[async_trait::async_trait]
pub trait Fetchable : Sync + Send {
	async fn fetch(&mut self, ctx: &crate::Context) -> Result<&mut Self, PullError>;
}

#[async_trait::async_trait]
impl Fetchable for apb::Node<serde_json::Value> {
	async fn fetch(&mut self, ctx: &crate::Context) -> Result<&mut Self, PullError> {
		if let apb::Node::Link(uri) = self {
			*self = crate::Context::request(Method::GET, uri.href(), None, ctx.base(), ctx.pkey(), ctx.domain())
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}

		Ok(self)
	}
}

// #[async_recursion::async_recursion]
// async fn crawl_replies(ctx: &crate::Context, id: &str, depth: usize) -> Result<(), PullError> {
// 	tracing::info!("crawling replies of '{id}'");
// 	let object = crate::Context::request(
// 		Method::GET, id, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
// 	).await?.json::<serde_json::Value>().await?;
// 
// 	let object_model = crate::model::object::Model::new(&object)?;
// 	match crate::model::object::Entity::insert(object_model.into_active_model())
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
// 			let replies = crate::Context::request(
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
// 		let replies = crate::Context::request(
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