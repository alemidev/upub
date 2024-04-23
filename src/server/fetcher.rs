use std::collections::BTreeMap;

use apb::{target::Addressed, Activity, Object};
use base64::Engine;
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{EntityTrait, IntoActiveModel};

use crate::{model, VERSION};

use super::{auth::HttpSignature, Context};

#[axum::async_trait]
pub trait Fetcher {
	async fn request(
		method: reqwest::Method,
		url: &str,
		payload: Option<&str>,
		from: &str,
		key: &str,
		domain: &str,
	) -> crate::Result<Response>;

	async fn fetch_user(&self, id: &str) -> crate::Result<model::user::Model>;
	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model>;
	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model>;
}


#[axum::async_trait]
impl Fetcher for Context {
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
		let payload_buf = payload.unwrap_or("").as_bytes();
		let digest = format!("sha-256={}", base64::prelude::BASE64_STANDARD.encode(openssl::sha::sha256(payload_buf)));

		let headers = vec!["(request-target)", "host", "date", "digest"];
		let headers_map : BTreeMap<String, String> = [
			("host".to_string(), host.clone()),
			("date".to_string(), date.clone()),
			("digest".to_string(), digest.clone()),
		].into();

		let mut signer = HttpSignature::new(
			format!("{from}#main-key"), // TODO don't hardcode #main-key
			"hs2019".to_string(),
			//"rsa-sha256".to_string(),
			&headers,
		);
		
		signer
			.build_manually(&method.to_string().to_lowercase(), &path, headers_map)
			.sign(key)?;

		Ok(reqwest::Client::new()
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
			.await?
			.error_for_status()?
		)
	}

	async fn fetch_user(&self, id: &str) -> crate::Result<model::user::Model> {
		if let Some(x) = model::user::Entity::find_by_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let user = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
		).await?.json::<serde_json::Value>().await?;
		let user_model = model::user::Model::new(&user)?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		model::user::Entity::insert(user_model.clone().into_active_model())
			.exec(self.db()).await?;

		Ok(user_model)
	}

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let activity = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
		).await?.json::<serde_json::Value>().await?;

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

		let addressed = activity.addressed();
		let activity_model = model::activity::Model::new(&activity)?;

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(self.db()).await?;

		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(Some(&activity_model.id), None, &expanded_addresses).await?;

		Ok(activity_model)
	}

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model> {
		fetch_object_inner(self, id, 0).await
	}
}

#[async_recursion::async_recursion]
async fn fetch_object_inner(ctx: &Context, id: &str, depth: usize) -> crate::Result<model::object::Model> {
	if let Some(x) = model::object::Entity::find_by_id(id).one(ctx.db()).await? {
		return Ok(x); // already in db, easy
	}

	let object = Context::request(
		Method::GET, id, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
	).await?.json::<serde_json::Value>().await?;

	if let Some(attributed_to) = object.attributed_to().id() {
		if let Err(e) = ctx.fetch_user(&attributed_to).await {
			tracing::warn!("could not get actor of fetched object: {e}");
		}
	}

	let addressed = object.addressed();
	let mut object_model = model::object::Model::new(&object)?;

	if let Some(reply) = &object_model.in_reply_to {
		if depth <= 16 {
			fetch_object_inner(ctx, reply, depth + 1).await?;
		} else {
			tracing::warn!("thread deeper than 16, giving up fetching more replies");
		}
	}

	// fix context also for remote posts
	// TODO this is not really appropriate because we're mirroring incorrectly remote objects, but
	// it makes it SOO MUCH EASIER for us to fetch threads and stuff, so we're filling it for them
	match (&object_model.in_reply_to, &object_model.context) {
		(Some(reply_id), None) => // get context from replied object
			object_model.context = fetch_object_inner(ctx, reply_id, depth + 1).await?.context,
		(None, None) => // generate a new context
			object_model.context = Some(crate::url!(ctx, "/context/{}", uuid::Uuid::new_v4().to_string())),
		(_, Some(_)) => {}, // leave it as set by user
	}

	for attachment in object.attachment() {
		let attachment_model = model::attachment::ActiveModel::new(&attachment, object_model.id.clone())?;
		model::attachment::Entity::insert(attachment_model)
			.exec(ctx.db())
			.await?;
	}

	let expanded_addresses = ctx.expand_addressing(addressed).await?;
	ctx.address_to(None, Some(&object_model.id), &expanded_addresses).await?;

	model::object::Entity::insert(object_model.clone().into_active_model())
		.exec(ctx.db()).await?;

	Ok(object_model)
}

#[axum::async_trait]
pub trait Fetchable : Sync + Send {
	async fn fetch(&mut self, ctx: &crate::server::Context) -> crate::Result<&mut Self>;
}

#[axum::async_trait]
impl Fetchable for apb::Node<serde_json::Value> {
	async fn fetch(&mut self, ctx: &crate::server::Context) -> crate::Result<&mut Self> {
		if let apb::Node::Link(uri) = self {
			let from = format!("{}{}", ctx.protocol(), ctx.domain()); // TODO helper to avoid this?
			let pkey = &ctx.app().private_key;
			*self = Context::request(Method::GET, uri.href(), None, &from, pkey, ctx.domain())
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}

		Ok(self)
	}
}
