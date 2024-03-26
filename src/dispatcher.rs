use base64::Engine;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect, SelectColumns};
use tokio::task::JoinHandle;

use crate::{activitypub::{activity::ap_activity, jsonld::LD, object::ap_object, user::outbox::UpubError}, activitystream::{object::activity::ActivityMut, Node}, model, VERSION};

pub struct Dispatcher;

impl Dispatcher {
	pub fn spawn(db: DatabaseConnection, domain: String, poll_interval: u64) -> JoinHandle<()> {
		tokio::spawn(async move {
			if let Err(e) = worker(db, domain, poll_interval).await {
				tracing::error!("delivery worker exited with error: {e}");
			}
		})
	}
}

async fn worker(db: DatabaseConnection, domain: String, poll_interval: u64) -> Result<(), UpubError> {
	let mut nosleep = true;
	loop {
		if nosleep { nosleep = false } else {
			tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
		}
		let Some(delivery) = model::delivery::Entity::find()
			.filter(Condition::all().add(model::delivery::Column::NotBefore.lte(chrono::Utc::now())))
			.order_by(model::delivery::Column::NotBefore, Order::Asc)
			.one(&db)
			.await?
		else { continue };

		let del_row = model::delivery::ActiveModel {
			id: sea_orm::ActiveValue::Set(delivery.id),
			..Default::default()
		};
		let del = model::delivery::Entity::delete(del_row)
			.exec(&db)
			.await?;

		if del.rows_affected == 0 {
			// another worker claimed this delivery
			nosleep = true;
			continue; // go back to the top
		}
		if delivery.expired() {
			// try polling for another one
			nosleep = true;
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

		let mut signer = Signer::new(MessageDigest::sha256(), &key)?;

		let without_protocol = delivery.target.replace("https://", "").replace("http://", "");
		let host = without_protocol.split('/').next().unwrap_or("").to_string();
		let request_target = without_protocol.replace(&host, "");
		let date = chrono::Utc::now().to_rfc2822();
		let signed_string = format!("(request-target): post {request_target}\nhost: {host}\ndate: {date}");
		tracing::info!("signing: \n{signed_string}");
		signer.update(signed_string.as_bytes())?;
		let signature = base64::prelude::BASE64_URL_SAFE.encode(signer.sign_to_vec()?);
		let signature_header = format!("keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date\",signature=\"{signature}\"", delivery.actor);
		tracing::info!("attaching header: {signature_header}");

		if let Err(e) = deliver(&delivery.target, payload, host, date, signature_header, &domain).await {
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

async fn deliver(target: &str, payload: serde_json::Value, host: String, date: String, signature_header: String, domain: &str) -> Result<(), reqwest::Error> {
	let res = reqwest::Client::new()
		.post(target)
		.json(&payload.ld_context())
		.header("Host", host)
		.header("Date", date)
		.header("Signature", signature_header)
		.header(CONTENT_TYPE, "application/ld+json")
		.header(USER_AGENT, format!("upub+{VERSION} ({domain})")) // TODO put instance admin email
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	tracing::info!("server answered with OK '{res}'");
	Ok(())
}

