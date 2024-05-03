use reqwest::Method;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder};
use tokio::{sync::broadcast, task::JoinHandle};

use apb::{ActivityMut, Node};
use crate::{errors::UpubError, model, routes::activitypub::jsonld::LD, server::{fetcher::Fetcher, Context}};

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
			.filter(model::delivery::Column::NotBefore.lte(chrono::Utc::now()))
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
			Some((activity, None)) => activity.ap().ld_context(),
			Some((activity, Some(object))) => {
				let always_embed = matches!(
					activity.activity_type,
					apb::ActivityType::Create
					| apb::ActivityType::Undo
					| apb::ActivityType::Update
					| apb::ActivityType::Accept(_)
					| apb::ActivityType::Reject(_)
				);
				if always_embed {
					activity.ap().set_object(Node::object(object.ap())).ld_context()
				} else {
					activity.ap().ld_context()
				}
			},
			None => {
				tracing::warn!("skipping dispatch for deleted object {}", delivery.activity);
				continue;
			},
		};

		let key = if delivery.actor == format!("https://{domain}") {
			let Some(model::application::Model { private_key: key, .. }) = model::application::Entity::find()
				.one(&db).await?
			else {
				tracing::error!("no private key configured for application");
				continue;
			};
			key
		} else {
			let Some(model::user::Model{ private_key: Some(key), .. }) = model::user::Entity::find_by_id(&delivery.actor)
				.one(&db).await?
			else { 
				tracing::error!("can not dispatch activity for user without private key: {}", delivery.actor);
				continue;
			};
			key
		};


		if let Err(e) = Context::request(
			Method::POST, &delivery.target,
			Some(&serde_json::to_string(&payload).unwrap()),
			&delivery.actor, &key, &domain
		).await {
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
