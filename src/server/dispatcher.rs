use base64::Engine;
use openssl::{hash::MessageDigest, pkey::{PKey, Private}, sign::Signer};
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder};
use tokio::{sync::broadcast, task::JoinHandle};

use apb::{ActivityMut, Node};
use crate::{routes::activitypub::{activity::ap_activity, object::ap_object}, errors::UpubError, model, server::Context, VERSION};

pub struct Dispatcher {
	waker: broadcast::Sender<()>,
}

impl Default for Dispatcher {
	fn default() -> Self {
		let (waker, _) = broadcast::channel(1);
		Dispatcher { waker }
	}
}

impl Dispatcher {
	pub fn new() -> Self { Dispatcher::default() }

	pub fn spawn(&self, db: DatabaseConnection, domain: String, poll_interval: u64) -> JoinHandle<()> {
		let waker = self.waker.subscribe();
		tokio::spawn(async move {
			if let Err(e) = worker(db, domain, poll_interval, waker).await {
				tracing::error!("delivery worker exited with error: {e}");
			}
		})
	}

	pub fn wakeup(&self) {
		match self.waker.send(()) {
			Err(_) => tracing::error!("no worker to wakeup"), 
			Ok(n) => tracing::debug!("woken {n} workers"),
		}
	}
}

async fn worker(db: DatabaseConnection, domain: String, poll_interval: u64, mut waker: broadcast::Receiver<()>) -> Result<(), UpubError> {
	loop {
		let Some(delivery) = model::delivery::Entity::find()
			.filter(Condition::all().add(model::delivery::Column::NotBefore.lte(chrono::Utc::now())))
			.order_by(model::delivery::Column::NotBefore, Order::Asc)
			.one(&db)
			.await?
		else {
			tokio::select! {
				biased;
				_ = waker.recv() => {},
				_ = tokio::time::sleep(std::time::Duration::from_secs(poll_interval)) => {},
			}
			continue
		};

		let del_row = model::delivery::ActiveModel {
			id: sea_orm::ActiveValue::Set(delivery.id),
			..Default::default()
		};
		let del = model::delivery::Entity::delete(del_row)
			.exec(&db)
			.await?;

		if del.rows_affected == 0 {
			// another worker claimed this delivery
			continue; // go back to the top
		}
		if delivery.expired() {
			// try polling for another one
			continue; // go back to top
		}

		tracing::info!("delivering {} to {}", delivery.activity, delivery.target);

		let payload = match model::activity::Entity::find_by_id(&delivery.activity)
			.find_also_related(model::object::Entity)
			.one(&db)
			.await? // TODO probably should not fail here and at least re-insert the delivery
		{
			Some((activity, Some(object))) => ap_activity(activity).set_object(Node::object(ap_object(object))),
			Some((activity, None)) => ap_activity(activity),
			None => {
				tracing::warn!("skipping dispatch for deleted object {}", delivery.activity);
				continue;
			},
		};

		let Some(model::user::Model{ private_key: Some(key), .. }) = model::user::Entity::find_by_id(&delivery.actor)
			.one(&db).await?
		else { 
			tracing::error!("can not dispatch activity for user without private key: {}", delivery.actor);
			continue;
		};

		let Ok(key) = PKey::private_key_from_pem(key.as_bytes())
		else {
			tracing::error!("failed parsing private key for user {}", delivery.actor);
			continue;
		};

		if let Err(e) = deliver(&key, &delivery.target, &delivery.actor, payload, &domain).await {
			tracing::warn!("failed delivery of {} to {} : {e}", delivery.activity, delivery.target);
			let new_delivery = model::delivery::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				not_before: sea_orm::ActiveValue::Set(delivery.next_delivery()),
				actor: sea_orm::ActiveValue::Set(delivery.actor),
				target: sea_orm::ActiveValue::Set(delivery.target),
				activity: sea_orm::ActiveValue::Set(delivery.activity),
				created: sea_orm::ActiveValue::Set(delivery.created),
				attempt: sea_orm::ActiveValue::Set(delivery.attempt + 1),
			};
			model::delivery::Entity::insert(new_delivery).exec(&db).await?;
		}
	}
}

async fn deliver(key: &PKey<Private>, to: &str, from: &str, payload: serde_json::Value, domain: &str) -> Result<(), UpubError> {
	let payload = serde_json::to_string(&payload).unwrap();
	let digest = format!("sha-256={}", base64::prelude::BASE64_STANDARD.encode(openssl::sha::sha256(payload.as_bytes())));
	let host = Context::server(to);
	let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(); // lmao @ "GMT"
	let path = to.replace("https://", "").replace("http://", "").replace(&host, "");

	// let headers : BTreeMap<String, String> = [
	// 	("Host".to_string(), host.clone()),
	// 	("Date".to_string(), date.clone()),
	// 	("Digest".to_string(), digest.clone()),
	// ].into();

	// let signature_header = Config::new()
	// 	.dont_use_created_field()
	// 	.require_header("host")
	// 	.require_header("date")
	// 	.require_header("digest")
	// 	.begin_sign("POST", &path, headers)
	// 	.unwrap()
	// 	.sign(format!("{from}#main-key"), |to_sign| {
	// 		tracing::info!("signing '{to_sign}'");
	// 		let mut signer = Signer::new(MessageDigest::sha256(), key)?;
	// 		signer.update(to_sign.as_bytes())?;
	// 		let signature = base64::prelude::BASE64_URL_SAFE.encode(signer.sign_to_vec()?);
	// 		Ok(signature) as Result<_, UpubError>
	// 	})
	// 	.unwrap()
	// 	.signature_header();
	
	let signature_header = {
		let to_sign = format!("(request-target): post {path}\nhost: {host}\ndate: {date}\ndigest: {digest}");
		let mut signer = Signer::new(MessageDigest::sha256(), key)?;
		signer.update(to_sign.as_bytes())?;
		let signature = base64::prelude::BASE64_STANDARD.encode(signer.sign_to_vec()?);
		format!("keyId=\"{from}#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{signature}\"")
	};

	reqwest::Client::new()
		.post(to)
		.header("Host", host)
		.header("Date", date)
		.header("Digest", digest)
		.header("Signature", signature_header)
		.header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
		.header(USER_AGENT, format!("upub+{VERSION} ({domain})")) // TODO put instance admin email
		.body(payload)
		.send()
		.await?
		.error_for_status()?;

	Ok(())
}
