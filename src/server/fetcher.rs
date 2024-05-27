use std::collections::BTreeMap;

use apb::{target::Addressed, Activity, Actor, ActorMut, Base, Collection, Object};
use base64::Engine;
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::EntityTrait;

use crate::{errors::UpubError, model, VERSION};

use super::{httpsign::HttpSignature, normalizer::Normalizer, Context};

#[axum::async_trait]
pub trait Fetcher {
	async fn webfinger(&self, user: &str, host: &str) -> crate::Result<String>;

	async fn fetch_user(&self, id: &str) -> crate::Result<model::actor::Model>;
	async fn pull_user(&self, id: &str) -> crate::Result<serde_json::Value>;

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model>;
	async fn pull_object(&self, id: &str) -> crate::Result<serde_json::Value>;

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model>;
	async fn pull_activity(&self, id: &str) -> crate::Result<serde_json::Value>;

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


	async fn fetch_user(&self, id: &str) -> crate::Result<model::actor::Model> {
		if let Some(x) = model::actor::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let user_document = self.pull_user(id).await?;
		let user_model = model::actor::ActiveModel::new(&user_document)?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		model::actor::Entity::insert(user_model).exec(self.db()).await?;
		
		// TODO fetch it back to get the internal id
		Ok(
			model::actor::Entity::find_by_ap_id(id)
				.one(self.db())
				.await?
				.ok_or_else(UpubError::internal_server_error)?
		)
	}

	async fn pull_user(&self, id: &str) -> crate::Result<serde_json::Value> {
		let mut user = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), self.pkey(), self.domain(),
		).await?.json::<serde_json::Value>().await?;

		// TODO try fetching these numbers from audience/generator fields to avoid making 2 more GETs
		if let Some(followers_url) = &user.followers().id() {
			let req = Self::request(
				Method::GET, followers_url, None,
				&format!("https://{}", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_followers) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_followers.total_items() {
						user = user.set_followers_count(Some(total));
					}
				}
			}
		}

		if let Some(following_url) = &user.following().id() {
			let req =  Self::request(
				Method::GET, following_url, None,
				&format!("https://{}", self.domain()), self.pkey(), self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_following) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_following.total_items() {
						user = user.set_following_count(Some(total));
					}
				}
			}
		}

		Ok(user)
	}

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_ap_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let activity_document = self.pull_activity(id).await?;
		let activity_model = self.insert_activity(activity_document, Some(Context::server(id))).await?;

		let addressed = activity_model.addressed();
		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(Some(activity_model.internal), None, &expanded_addresses).await?;

		Ok(activity_model)
	}

	async fn pull_activity(&self, id: &str) -> crate::Result<serde_json::Value> {
		let activity = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), self.pkey(), self.domain(),
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

		Ok(activity)
	}

	async fn fetch_thread(&self, _id: &str) -> crate::Result<()> {
		// crawl_replies(self, id, 0).await
		todo!()
	}

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model> {
		fetch_object_inner(self, id, 0).await
	}

	async fn pull_object(&self, id: &str) -> crate::Result<serde_json::Value> {
		Ok(
			Context::request(
				Method::GET, id, None, &format!("https://{}", self.domain()), self.pkey(), self.domain(),
			)
				.await?
				.json::<serde_json::Value>()
				.await?
		)
	}
}

#[async_recursion::async_recursion]
async fn fetch_object_inner(ctx: &Context, id: &str, depth: usize) -> crate::Result<model::object::Model> {
	if let Some(x) = model::object::Entity::find_by_ap_id(id).one(ctx.db()).await? {
		return Ok(x); // already in db, easy
	}

	let object = ctx.pull_object(id).await?;

	if let Some(oid) = object.id() {
		if oid != id {
			if let Some(x) = model::object::Entity::find_by_ap_id(oid).one(ctx.db()).await? {
				return Ok(x); // already in db, but with id different that given url
			}
		}
	}

	if let Some(attributed_to) = object.attributed_to().id() {
		if let Err(e) = ctx.fetch_user(&attributed_to).await {
			tracing::warn!("could not get actor of fetched object: {e}");
		}
	}

	let addressed = object.addressed();

	if let Some(reply) = object.in_reply_to().id() {
		if depth <= 16 {
			fetch_object_inner(ctx, &reply, depth + 1).await?;
		} else {
			tracing::warn!("thread deeper than 16, giving up fetching more replies");
		}
	}

	let object_model = ctx.insert_object(object, None).await?;

	let expanded_addresses = ctx.expand_addressing(addressed).await?;
	ctx.address_to(None, Some(object_model.internal), &expanded_addresses).await?;

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
