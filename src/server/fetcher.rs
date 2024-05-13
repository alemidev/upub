use std::collections::BTreeMap;

use apb::{target::Addressed, Activity, Base, Collection, CollectionPage, Link, Object};
use base64::Engine;
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{errors::UpubError, model, VERSION};

use super::{auth::HttpSignature, Context};

#[axum::async_trait]
pub trait Fetcher {
	async fn webfinger(&self, user: &str, host: &str) -> crate::Result<String>;

	async fn fetch_user(&self, id: &str) -> crate::Result<model::user::Model>;
	async fn pull_user(&self, id: &str) -> crate::Result<model::user::Model>;

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model>;
	async fn pull_object(&self, id: &str) -> crate::Result<model::object::Model>;

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model>;
	async fn pull_activity(&self, id: &str) -> crate::Result<model::activity::Model>;

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


	async fn fetch_user(&self, id: &str) -> crate::Result<model::user::Model> {
		if let Some(x) = model::user::Entity::find_by_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let user_model = self.pull_user(id).await?;

		// TODO this may fail: while fetching, remote server may fetch our service actor.
		//      if it does so with http signature, we will fetch that actor in background
		//      meaning that, once we reach here, it's already inserted and returns an UNIQUE error
		model::user::Entity::insert(user_model.clone().into_active_model())
			.exec(self.db()).await?;

		Ok(user_model)
	}

	async fn pull_user(&self, id: &str) -> crate::Result<model::user::Model> {
		let user = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
		).await?.json::<serde_json::Value>().await?;
		let mut user_model = model::user::Model::new(&user)?;

		// TODO try fetching these numbers from audience/generator fields to avoid making 2 more GETs
		if let Some(followers_url) = &user_model.followers {
			let req = Self::request(
				Method::GET, followers_url, None,
				&format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_followers) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_followers.total_items() {
						user_model.followers_count = total as i64;
					}
				}
			}
		}

		if let Some(following_url) = &user_model.following {
			let req =  Self::request(
				Method::GET, following_url, None,
				&format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
			).await;
			if let Ok(res) = req {
				if let Ok(user_following) = res.json::<serde_json::Value>().await {
					if let Some(total) = user_following.total_items() {
						user_model.following_count = total as i64;
					}
				}
			}
		}

		Ok(user_model)
	}

	async fn fetch_activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_id(id).one(self.db()).await? {
			return Ok(x); // already in db, easy
		}

		let activity_model = self.pull_activity(id).await?;

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(self.db()).await?;

		let addressed = activity_model.addressed();
		let expanded_addresses = self.expand_addressing(addressed).await?;
		self.address_to(Some(&activity_model.id), None, &expanded_addresses).await?;

		Ok(activity_model)
	}

	async fn pull_activity(&self, id: &str) -> crate::Result<model::activity::Model> {
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

		let activity_model = model::activity::Model::new(&activity)?;

		Ok(activity_model)
	}

	async fn fetch_thread(&self, id: &str) -> crate::Result<()> {
		crawl_replies(self, id, 0).await
	}

	async fn fetch_object(&self, id: &str) -> crate::Result<model::object::Model> {
		fetch_object_inner(self, id, 0).await
	}

	async fn pull_object(&self, id: &str) -> crate::Result<model::object::Model> {
		let object = Context::request(
			Method::GET, id, None, &format!("https://{}", self.domain()), &self.app().private_key, self.domain(),
		).await?.json::<serde_json::Value>().await?;

		Ok(model::object::Model::new(&object)?)
	}
}

#[async_recursion::async_recursion]
async fn crawl_replies(ctx: &Context, id: &str, depth: usize) -> crate::Result<()> {
	tracing::info!("crawling replies of '{id}'");
	let object = Context::request(
		Method::GET, id, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
	).await?.json::<serde_json::Value>().await?;

	let object_model = model::object::Model::new(&object)?;
	match model::object::Entity::insert(object_model.into_active_model())
		.exec(ctx.db()).await
	{
		Ok(_) => {},
		Err(sea_orm::DbErr::RecordNotInserted) => {},
		Err(sea_orm::DbErr::Exec(_)) => {}, // ughhh bad fix for sqlite
		Err(e) => return Err(e.into()),
	}

	if depth > 16 {
		tracing::warn!("stopping thread crawling: too deep!");
		return Ok(());
	}

	let mut page_url = match object.replies().get() {
		Some(serde_json::Value::String(x)) => {
			let replies = Context::request(
				Method::GET, x, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
			).await?.json::<serde_json::Value>().await?;
			replies.first().id()
		},
		Some(serde_json::Value::Object(x)) => {
			let obj = serde_json::Value::Object(x.clone()); // lol putting it back, TODO!
			obj.first().id()
		},
		_ => return Ok(()),
	};

	while let Some(ref url) = page_url {
		let replies = Context::request(
			Method::GET, url, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
		).await?.json::<serde_json::Value>().await?;

		for reply in replies.items() {
			// TODO right now it crawls one by one, could be made in parallel but would be quite more
			// abusive, so i'll keep it like this while i try it out
			crawl_replies(ctx, reply.href(), depth + 1).await?;
		}

		page_url = replies.next().id();
	}

	Ok(())
}

#[async_recursion::async_recursion]
async fn fetch_object_inner(ctx: &Context, id: &str, depth: usize) -> crate::Result<model::object::Model> {
	if let Some(x) = model::object::Entity::find_by_id(id).one(ctx.db()).await? {
		return Ok(x); // already in db, easy
	}

	let object = Context::request(
		Method::GET, id, None, &format!("https://{}", ctx.domain()), &ctx.app().private_key, ctx.domain(),
	).await?.json::<serde_json::Value>().await?;

	if let Some(oid) = object.id() {
		if oid != id {
			if let Some(x) = model::object::Entity::find_by_id(oid).one(ctx.db()).await? {
				return Ok(x); // already in db, but with id different that given url
			}
		}
	}

	if let Some(attributed_to) = object.attributed_to().id() {
		if let Err(e) = ctx.fetch_user(&attributed_to).await {
			tracing::warn!("could not get actor of fetched object: {e}");
		}
		model::user::Entity::update_many()
			.col_expr(model::user::Column::StatusesCount, Expr::col(model::user::Column::StatusesCount).add(1))
			.filter(model::user::Column::Id.eq(&attributed_to))
			.exec(ctx.db())
			.await?;
	}

	let addressed = object.addressed();
	let mut object_model = model::object::Model::new(&object)?;

	if let Some(reply) = &object_model.in_reply_to {
		if depth <= 16 {
			fetch_object_inner(ctx, reply, depth + 1).await?;
			model::object::Entity::update_many()
				.filter(model::object::Column::Id.eq(reply))
				.col_expr(model::object::Column::Comments, Expr::col(model::object::Column::Comments).add(1))
				.exec(ctx.db())
				.await?;
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
		let attachment_model = model::attachment::ActiveModel::new(&attachment, object_model.id.clone(), None)?;
		model::attachment::Entity::insert(attachment_model)
			.exec(ctx.db())
			.await?;
	}
	// lemmy sends us an image field in posts, treat it like an attachment i'd say
	if let Some(img) = object.image().get() {
		// TODO lemmy doesnt tell us the media type but we use it to display the thing...
		let img_url = img.url().id().unwrap_or_default();
		let media_type = if img_url.ends_with("png") {
			Some("image/png".to_string())
		} else if img_url.ends_with("webp") {
			Some("image/webp".to_string())
		} else if img_url.ends_with("jpeg") || img_url.ends_with("jpg") {
			Some("image/jpeg".to_string())
		} else {
			None
		};
		let attachment_model = model::attachment::ActiveModel::new(img, object_model.id.clone(), media_type)?;
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
