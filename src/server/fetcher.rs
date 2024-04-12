use std::collections::BTreeMap;

use base64::Engine;
use http_signature_normalization::Config;
use openssl::{hash::MessageDigest, pkey::{PKey, Private}, sign::Signer};
use reqwest::{header::{CONTENT_TYPE, USER_AGENT}, Method};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

use crate::{VERSION, model};

use super::Context;


pub struct Fetcher {
	db: DatabaseConnection,
	key: PKey<Private>, // TODO store pre-parsed
	domain: String, // TODO merge directly with Context so we don't need to copy this
}

impl Fetcher {
	pub fn new(db: DatabaseConnection, domain: String, key: String) -> Self {
		Fetcher { db, domain, key: PKey::private_key_from_pem(key.as_bytes()).unwrap() }
	}

	pub async fn request<T: serde::de::DeserializeOwned>(
		method: reqwest::Method,
		url: &str,
		payload: Option<&str>,
		from: &str,
		key: &PKey<Private>,
		domain: &str,
	) -> reqwest::Result<T> {
		let host = Context::server(url);
		let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(); // lmao @ "GMT"
		let path = url.replace("https://", "").replace("http://", "").replace(&host, "");
		let mut headers : BTreeMap<String, String> = [
			("Host".to_string(), host.clone()),
			("Date".to_string(), date.clone()),
		].into();

		let mut client = 
			reqwest::Client::new()
				.request(method, url)
				.header("Host", host)
				.header("Date", date);

		let mut signature_cfg = Config::new();

		if let Some(payload) = payload {
			let digest = format!("sha-256={}", base64::prelude::BASE64_STANDARD.encode(openssl::sha::sha256(payload.as_bytes())));
			headers.insert("Digest".to_string(), digest.clone());
			signature_cfg = signature_cfg.require_header("digest");
			client = client
				.header("Digest", digest)
				.body(payload.to_string());
		}

		let signature_header = signature_cfg
			.dont_use_created_field()
			.require_header("host")
			.require_header("date")
			.begin_sign("POST", &path, headers)
			.unwrap()
			.sign(format!("{from}#main-key"), |to_sign| {
				tracing::info!("signing '{to_sign}'");
				let mut signer = Signer::new(MessageDigest::sha256(), key)?;
				signer.update(to_sign.as_bytes())?;
				let signature = base64::prelude::BASE64_URL_SAFE.encode(signer.sign_to_vec()?);
				Ok(signature) as crate::Result<_>
			})
			.unwrap()
			.signature_header();

		client
			.header("Signature", signature_header)
			.header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
			.header(USER_AGENT, format!("upub+{VERSION} ({domain})")) // TODO put instance admin email
			.send()
			.await?
			.error_for_status()?
			.json()
			.await
	}

	pub async fn user(&self, id: &str) -> crate::Result<model::user::Model> {
		if let Some(x) = model::user::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let user = Self::request::<serde_json::Value>(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?;
		let user_model = model::user::Model::new(&user)?;

		model::user::Entity::insert(user_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(user_model)
	}

	pub async fn activity(&self, id: &str) -> crate::Result<model::activity::Model> {
		if let Some(x) = model::activity::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let activity = Self::request::<serde_json::Value>(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?;
		let activity_model = model::activity::Model::new(&activity)?;

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(activity_model)
	}

	pub async fn object(&self, id: &str) -> crate::Result<model::object::Model> {
		if let Some(x) = model::object::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		let object = Self::request::<serde_json::Value>(
			Method::GET, id, None, &format!("https://{}", self.domain), &self.key, &self.domain,
		).await?;
		let object_model = model::object::Model::new(&object)?;

		model::object::Entity::insert(object_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(object_model)
	}
}
