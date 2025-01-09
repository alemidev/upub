use std::collections::BTreeMap;

use apb::{Shortcuts, Activity, Actor, ActorMut, Base, Collection, CollectionPage, Object};
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, NotSet, QueryFilter, ActiveModelTrait};

use super::{Addresser, Cloaker, Normalizer};
use httpsign::HttpSignature;

use crate::AP;

#[derive(Debug, Clone)]
pub enum Pull<T> {
	Actor(T),
	Activity(T),
	Object(T),
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
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

	#[error("request aborted due to configured policies")]
	AbortedForPolicy,

	#[error("error constructing http signature: {0:?}")]
	HttpSignature(#[from] httpsign::HttpSignatureError),
}

impl RequestError {
	fn mismatch(expected: apb::ObjectType, found: apb::ObjectType) -> Self {
		RequestError::Mismatch(expected, found)
	}
}

impl Pull<serde_json::Value> {
	pub fn actor(self) -> Result<serde_json::Value, RequestError> {
		match self {
			Self::Actor(x) => Ok(x),
			Self::Activity(x) => Err(RequestError::mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Err(RequestError::mismatch(apb::ObjectType::Actor(apb::ActorType::Person), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn activity(self) -> Result<serde_json::Value, RequestError> {
		match self {
			Self::Actor(x) => Err(RequestError::mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Ok(x),
			Self::Object(x) => Err(RequestError::mismatch(apb::ObjectType::Activity(apb::ActivityType::Activity), x.object_type().unwrap_or(apb::ObjectType::Object))),
		}
	}

	pub fn object(self) -> Result<serde_json::Value, RequestError> {
		match self {
			Self::Actor(x) => Err(RequestError::mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Actor(apb::ActorType::Person)))),
			Self::Activity(x) => Err(RequestError::mismatch(apb::ObjectType::Object, x.object_type().unwrap_or(apb::ObjectType::Activity(apb::ActivityType::Activity)))),
			Self::Object(x) => Ok(x),
		}
	}
}

#[async_trait::async_trait]
pub trait Fetcher {
	async fn pull(&self, id: &str) -> Result<Pull<serde_json::Value>, RequestError> { self.pull_r(id, 0).await }
	async fn pull_r(&self, id: &str, depth: u32) -> Result<Pull<serde_json::Value>, RequestError>;


	async fn webfinger(&self, user: &str, host: &str) -> Result<Option<String>, RequestError>;

	async fn fetch_domain(&self, domain: &str, tx: &impl ConnectionTrait) -> Result<crate::model::instance::Model, RequestError>;

	async fn fetch_user(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::actor::Model, RequestError>;
	async fn resolve_user(&self, actor: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::actor::ActiveModel, RequestError>;

	async fn fetch_activity(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, RequestError>;
	async fn resolve_activity(&self, activity: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, RequestError>;

	async fn fetch_object(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError>;
	async fn resolve_object(&self, object: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError>;

	async fn fetch_thread(&self, id: &str, tx: &impl ConnectionTrait) -> Result<(), RequestError>;
	async fn fetch_outbox(&self, id: &str, tx: &impl ConnectionTrait) -> Result<(), RequestError>;

	fn client(domain: &str) -> reqwest::Client {
		reqwest::Client::builder()
			.user_agent(format!("upub+{} ({domain})", crate::VERSION))
			.connect_timeout(std::time::Duration::from_secs(30)) // TODO may be cool to configure these
			.read_timeout(std::time::Duration::from_secs(30)) // TODO may be cool to configure these
			.timeout(std::time::Duration::from_secs(300)) // TODO may be cool to configure these
			.build()
			.expect("failed building http client, check system tls or resolver")
	}

	async fn request(
		method: reqwest::Method,
		url: &str,
		payload: Option<&str>,
		from: &str,
		key: &str,
		domain: &str,
	) -> Result<Response, RequestError> {
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
			//"hs2019".to_string(), // TODO could we switch to this now?
			"rsa-sha256".to_string(),
			&headers,
		);
		
		signer
			.build_manually(&method.to_string().to_lowercase(), &path, headers_map)
			.sign(key)?;

		let response = Self::client(domain)
			.request(method, url)
			.header(ACCEPT, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
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
				Err(RequestError::Fetch(
					e.status().unwrap_or_default(),
					response.text().await?,
				)),
		}
	}
}


#[async_trait::async_trait]
impl Fetcher for crate::Context {
	async fn pull_r(&self, id: &str, depth: u32) -> Result<Pull<serde_json::Value>, RequestError> {
		if crate::ext::is_blacklisted(id, &self.cfg().reject.fetch) {
			return Err(RequestError::AbortedForPolicy);
		}

		tracing::debug!("fetching {id}");
		// let _domain = self.fetch_domain(&crate::Context::server(id)).await?;

		let document = Self::request(
			Method::GET, id, None,
			self.base(), self.pkey(), self.domain(),
		)
			.await?
			.json::<serde_json::Value>()
			.await?;

		let doc_id = document.id()?;
		if id != doc_id {
			if depth >= self.cfg().security.max_id_redirects {
				return Err(RequestError::TooManyRedirects);
			}
			return self.pull(&doc_id).await;
		}

		crate::downtime::unset(self.db(), &crate::Context::server(id)).await?;

		match document.object_type()? {
			apb::ObjectType::Collection(x) => Err(RequestError::mismatch(apb::ObjectType::Object, apb::ObjectType::Collection(x))),
			apb::ObjectType::Tombstone => Err(RequestError::Tombstone),
			apb::ObjectType::Activity(_) => Ok(Pull::Activity(document)),
			apb::ObjectType::Actor(_) => Ok(Pull::Actor(document)),
			_ => Ok(Pull::Object(document)),
		}
	}


	async fn webfinger(&self, user: &str, host: &str) -> Result<Option<String>, RequestError> {
		// TODO username:host is not guaranteed unique!!!! either return a Vec<String> or maybe specify
		//      which kind of actor??
		if let Some(usr) = crate::model::actor::Entity::find()
			.filter(crate::model::actor::Column::PreferredUsername.eq(user))
			.filter(crate::model::actor::Column::Domain.eq(host))
			.one(self.db())
			.await?
		{
			return Ok(Some(usr.id));
		}

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

	async fn fetch_domain(&self, domain: &str, tx: &impl ConnectionTrait) -> Result<crate::model::instance::Model, RequestError> {
		if let Some(x) = crate::model::instance::Entity::find_by_domain(domain).one(tx).await? {
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
			self.base(), self.pkey(), self.domain(),
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
		crate::model::instance::Entity::insert(active_model).exec(tx).await?;
		let internal = crate::model::instance::Entity::domain_to_internal(domain, tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(domain.to_string()))?;
		instance_model.internal = internal;

		Ok(instance_model)
	}

	async fn resolve_user(&self, mut document: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::actor::ActiveModel, RequestError> {
		let id = document.id()?.to_string();

		let _domain = self.fetch_domain(&crate::Context::server(&id), tx).await?;

		// TODO try fetching these numbers from audience/generator fields to avoid making 2 more GETs every time
		if document.followers_count().is_err() {
			if let Ok(followers) = document.followers().resolve(self).await {
				if let Ok(total) = followers.total_items() {
					document = document.set_followers_count(Some(total));
				}
			}
		}

		if document.following_count().is_err() {
			if let Ok(following) = document.following().resolve(self).await {
				if let Ok(total) = following.total_items() {
					document = document.set_following_count(Some(total));
				}
			}
		}

		if document.statuses_count().is_err() {
			if let Ok(outbox) = document.outbox().resolve(self).await {
				if let Ok(total) = outbox.total_items() {
					document = document.set_statuses_count(Some(total));
				}
			}
		}

		let mut user_model = AP::actor_q(&document, None)?;

		// cloak remote images
		if let Set(Some(ref image)) = user_model.image {
			if !image.starts_with(self.base()) {
				user_model.image = Set(Some(self.cloaked(image)));
			}
		}

		if let Set(Some(ref icon)) = user_model.icon {
			if !icon.starts_with(self.base()) {
				user_model.icon = Set(Some(self.cloaked(icon)));
			}
		}

		Ok(user_model)
	}

	async fn fetch_user(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::actor::Model, RequestError> {
		if let Some(x) = crate::model::actor::Entity::find_by_ap_id(id).one(tx).await? {
			return Ok(x); // already in db, easy
		}

		let document = self.pull(id).await?.actor()?;

		if document.id()? != id {
			if let Some(x) = crate::model::actor::Entity::find_by_ap_id(&document.id()?).one(tx).await? {
				return Ok(x); // already in db but we had to follow the "pretty" url, mehh
			}
		}

		let active_model = self.resolve_user(document, tx).await?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		crate::model::actor::Entity::insert(active_model).exec(tx).await?;
		
		// TODO fetch it back to get the internal id
		Ok(
			crate::model::actor::Entity::find_by_ap_id(id)
				.one(tx)
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(id.to_string()))?
		)
	}

	async fn fetch_activity(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, RequestError> {
		if let Some(x) = crate::model::activity::Entity::find_by_ap_id(id).one(tx).await? {
			return Ok(x); // already in db, easy
		}

		let activity = self.pull(id).await?.activity()?;

		self.resolve_activity(activity, tx).await
	}

	async fn resolve_activity(&self, activity: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, RequestError> {
		let _domain = self.fetch_domain(&crate::Context::server(&activity.id()?), tx).await?;

		if let Ok(activity_actor) = activity.actor().id() {
			if let Err(e) = self.fetch_user(&activity_actor, tx).await {
				tracing::warn!("could not get actor of fetched activity: {e}");
			}
		}

		if let Ok(activity_object) = activity.object().id() {
			if let Err(e) = self.fetch_object(&activity_object, tx).await {
				tracing::warn!("could not get object of fetched activity: {e}");
			}
		}

		let activity_model = self.insert_activity(activity, tx).await?;
		self.address(Some(&activity_model), None, tx).await?;

		Ok(activity_model)
	}

	async fn fetch_thread(&self, id: &str, tx: &impl ConnectionTrait) -> Result<(), RequestError> {
		let object = self.pull(id).await?.object()?;

		// also update object stats since we're pulling it again
		let model = self.fetch_object(id, tx).await?;
		let mut active = model.clone().into_active_model();
		let mut changed = false;

		let new_like_count = object.likes_count().unwrap_or_default();
		if new_like_count > model.likes {
			active.likes = Set(new_like_count);
			changed = true;
		}

		let new_share_count = object.shares_count().unwrap_or_default();
		if new_share_count > model.announces {
			active.announces = Set(new_share_count);
			changed = true;
		}

		if changed {
			active.update(tx).await?;
		}

		// crawl replies collection
		let replies = object.replies().resolve(self).await?;

		let mut page;
		let mut next = replies.first();

		loop {
			page = next.resolve(self).await?;

			// fix for mastodon: at some point it introduces ?only_other_accounts=true and then returns a
			// collection, not a page anymore ???
			if matches!(page.object_type()?, apb::ObjectType::Collection(apb::CollectionType::Collection)) {
				page = page.first().into_inner()?;
			}

			// TODO parallelize these

			for obj in page.items().flat() {
				if let Err(e) = self.fetch_object(&obj.id()?, tx).await {
					tracing::warn!("error fetching reply: {e}");
				}
			}

			for obj in page.ordered_items().flat() {
				if let Err(e) = self.fetch_object(&obj.id()?, tx).await {
					tracing::warn!("error fetching reply: {e}");
				}
			}

			next = page.next();
			if next.is_empty() { break };
		}

		Ok(())
	}

	async fn fetch_outbox(&self, id: &str, tx: &impl ConnectionTrait) -> Result<(), RequestError> {
		let actor = self.pull(id).await?.actor()?;
		let outbox = actor
			.outbox().resolve(self).await?
			.first().resolve(self).await?;

		// TODO parallelize these

		for item in outbox.ordered_items().all_ids() {
			self.fetch_activity(&item, tx).await?;
		}

		Ok(())
	}

	async fn fetch_object(&self, id: &str, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError> {
		fetch_object_r(self, id, 0, tx).await
	}

	async fn resolve_object(&self, object: serde_json::Value, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError> {
		resolve_object_r(self, object, 0, tx).await
	}
}

#[async_recursion::async_recursion]
async fn fetch_object_r(ctx: &crate::Context, id: &str, depth: u32, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError> {
	if let Some(x) = crate::model::object::Entity::find_by_ap_id(id).one(tx).await? {
		return Ok(x); // already in db, easy
	}

	let object = ctx.pull(id).await?.object()?;

	if object.id()? != id {
		if let Some(x) = crate::model::object::Entity::find_by_ap_id(&object.id()?).one(tx).await? {
			return Ok(x); // already in db but we had to follow the "pretty" url, mehh
		}
	}

	resolve_object_r(ctx, object, depth, tx).await
}

async fn resolve_object_r(ctx: &crate::Context, object: serde_json::Value, depth: u32, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, RequestError> {
	let id = object.id()?.to_string();

	if let Ok(oid) = object.id() {
		if oid != id {
			if let Some(x) = crate::model::object::Entity::find_by_ap_id(&oid).one(tx).await? {
				return Ok(x); // already in db, but with id different that given url
			}
		}
	}

	if let Ok(attributed_to) = object.attributed_to().id() {
		if let Err(e) = ctx.fetch_user(&attributed_to, tx).await {
			tracing::warn!("could not get actor of fetched object: {e}");
		}
	}

	if let Ok(reply) = object.in_reply_to().id() {
		if depth <= ctx.cfg().security.thread_crawl_depth {
			fetch_object_r(ctx, &reply, depth + 1, tx).await?;
		} else {
			tracing::warn!("thread deeper than {}, giving up fetching more replies", ctx.cfg().security.thread_crawl_depth);
		}
	}

	let object_model = ctx.insert_object(object, tx).await?;
	ctx.address(None, Some(&object_model), tx).await?;

	Ok(object_model)
}

#[allow(async_fn_in_trait)]
pub trait Dereferenceable<T> : Sync + Send {
	async fn resolve(self, ctx: &crate::Context) -> Result<T, RequestError>;
}

impl Dereferenceable<serde_json::Value> for apb::Node<serde_json::Value> {
	async fn resolve(self, ctx: &crate::Context) -> Result<serde_json::Value, RequestError> {
		match self {
			apb::Node::Link(uri) => {
				let href = uri.href()?;
				if crate::ext::is_blacklisted(&href, &ctx.cfg().reject.fetch) {
					return Err(RequestError::AbortedForPolicy);
				}
				tracing::info!("dereferencing {href}");
				let res = crate::Context::request(Method::GET, &href, None, ctx.base(), ctx.pkey(), ctx.domain())
					.await?
					.json::<serde_json::Value>()
					.await?;
				Ok(res)
			},
			apb::Node::Object(x) => Ok(*x),
			apb::Node::Empty => Err(RequestError::Tombstone),
			apb::Node::Array(_) => Err(RequestError::Malformed(apb::FieldErr("id"))), // TODO weird!!
		}
	}
}
