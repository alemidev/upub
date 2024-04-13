use std::collections::BTreeMap;

use base64::Engine;
use reqwest::{header::{ACCEPT, CONTENT_TYPE, USER_AGENT}, Method, Response};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

use crate::{model, VERSION};

use super::{auth::HttpSignature, Context};


pub struct Fetcher {
	db: DatabaseConnection,
	key: String, // TODO store pre-parsed
	domain: String, // TODO merge directly with Context so we don't need to copy this
}

impl Fetcher {
	pub fn new(db: DatabaseConnection, domain: String, key: String) -> Self {
		Fetcher { db, domain, key }
	}

	pub async fn request(
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

		let mut headers = vec!["(request-target)", "host", "date"];
		let mut headers_map : BTreeMap<String, String> = [
			("host".to_string(), host.clone()),
			("date".to_string(), date.clone()),
		].into();

		let mut client = reqwest::Client::new()
			.request(method.clone(), url)
			.header(ACCEPT, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(USER_AGENT, format!("upub+{VERSION} ({domain})"))
			.header("Host", host.clone())
			.header("Date", date.clone());


		if let Some(payload) = payload {
			let digest = format!("sha-256={}", base64::prelude::BASE64_STANDARD.encode(openssl::sha::sha256(payload.as_bytes())));
			headers_map.insert("digest".to_string(), digest.clone());
			headers.push("digest");
			client = client
				.header("Digest", digest)
				.body(payload.to_string());
		}

		let mut signer = HttpSignature::new(
			format!("{from}#main-key"), // TODO don't hardcode #main-key
			"rsa-sha256".to_string(),
			&headers,
		);
		
		signer
			.build_manually(&method.to_string().to_lowercase(), &path, headers_map)
			.sign(key)?;

		let res = client
				.header("Signature", signer.header())
				.send()
				.await?;

		Ok(res.error_for_status()?)
	}

	pub async fn user(&self, id: &str) -> crate::Result<model::user::Model> {
		if let Some(x) = model::user::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let user = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?.json::<serde_json::Value>().await?;
		let user_model = model::user::Model::new(&user)?;

		model::user::Entity::insert(user_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(user_model)
	}

	pub async fn activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let activity = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?.json::<serde_json::Value>().await?;
		let activity_model = model::activity::Model::new(&activity)?;

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(activity_model)
	}

	pub async fn object(&self, id: &str) -> crate::Result<model::object::Model> {
		if let Some(x) = model::object::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let object = Self::request(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?.json::<serde_json::Value>().await?;
		let object_model = model::object::Model::new(&object)?;

		model::object::Entity::insert(object_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(object_model)
	}
}
