use sea_orm::{ActiveValue::{NotSet, Set}, DbErr, EntityTrait};

use crate::traits::fetch::Fetcher;

#[async_trait::async_trait]
pub trait Addresser {
	async fn expand_addressing(&self, targets: Vec<String>) -> Result<Vec<String>, DbErr>;
	async fn address_to(&self, aid: Option<i64>, oid: Option<i64>, targets: &[String]) -> Result<(), DbErr>;
	async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> Result<(), DbErr>;
	//#[deprecated = "should probably directly invoke address_to() since we most likely have internal ids at this point"]
	async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> Result<(), DbErr>;
}

#[async_trait::async_trait]
impl Addresser for crate::Context {
	async fn expand_addressing(&self, targets: Vec<String>) -> Result<Vec<String>, DbErr> {
		let mut out = Vec::new();
		for target in targets {
			if target.ends_with("/followers") {
				let target_id = target.replace("/followers", "");
				let mut followers = crate::model::relation::Entity::followers(&target_id, self.db())
					.await?
					.unwrap_or_else(Vec::new);
				if followers.is_empty() { // stuff with zero addressing will never be seen again!!! TODO
					followers.push(target_id);
				}
				for follower in followers {
					out.push(follower);
				}
			} else {
				out.push(target);
			}
		}
		Ok(out)
	}

	async fn address_to(&self, aid: Option<i64>, oid: Option<i64>, targets: &[String]) -> Result<(), DbErr> {
		// TODO address_to became kind of expensive, with these two selects right away and then another
		//      select for each target we're addressing to... can this be improved??
		let local_activity = if let Some(x) = aid { self.is_local_internal_activity(x).await.unwrap_or(false) } else { false };
		let local_object = if let Some(x) = oid { self.is_local_internal_object(x).await.unwrap_or(false) } else { false };
		let mut addressing = Vec::new();
		for target in targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| !to.ends_with("/followers"))
			.filter(|to| local_activity || local_object || to.as_str() == apb::target::PUBLIC || self.is_local(to))
		{
			let (server, actor) = if target == apb::target::PUBLIC { (None, None) } else {
				match (
					crate::model::instance::Entity::domain_to_internal(&crate::Context::server(target), self.db()).await?,
					crate::model::actor::Entity::ap_to_internal(target, self.db()).await?,
				) {
					(Some(server), Some(actor)) => (Some(server), Some(actor)),
					(None, _) => { tracing::error!("failed resolving domain"); continue; },
					(_, None) => { tracing::error!("failed resolving actor"); continue; },
				}
			};
			addressing.push(
				crate::model::addressing::ActiveModel {
					internal: NotSet,
					instance: Set(server),
					actor: Set(actor),
					activity: Set(aid),
					object: Set(oid),
					published: Set(chrono::Utc::now()),
				}
			);
		}

		if !addressing.is_empty() {
			crate::model::addressing::Entity::insert_many(addressing)
				.exec(self.db())
				.await?;
		}

		Ok(())
	}

	async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> Result<(), DbErr> {
		let mut deliveries = Vec::new();
		for target in targets.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| crate::Context::server(to) != self.domain())
			.filter(|to| to != &apb::target::PUBLIC)
		{
			// TODO fetch concurrently
			match self.fetch_user(target).await {
				Ok(crate::model::actor::Model { inbox: Some(inbox), .. }) => deliveries.push(
					crate::model::delivery::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						actor: Set(from.to_string()),
						// TODO we should resolve each user by id and check its inbox because we can't assume
						// it's /actors/{id}/inbox for every software, but oh well it's waaaaay easier now
						target: Set(inbox),
						activity: Set(aid.to_string()),
						published: Set(chrono::Utc::now()),
						not_before: Set(chrono::Utc::now()),
						attempt: Set(0),
					}
				),
				Ok(_) => tracing::error!("resolved target but missing inbox: '{target}', skipping delivery"),
				Err(e) => tracing::error!("failed resolving target inbox: {e}, skipping delivery to '{target}'"),
			}
		}

		if !deliveries.is_empty() {
			crate::model::delivery::Entity::insert_many(deliveries)
				.exec(self.db())
				.await?;
		}

		// TODO can we make deliveries instant? for better UX
		// self.dispatcher().wakeup();

		Ok(())
	}

	//#[deprecated = "should probably directly invoke address_to() since we most likely have internal ids at this point"]
	async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> Result<(), DbErr> {
		let addressed = self.expand_addressing(activity_targets).await?;
		let internal_aid = crate::model::activity::Entity::ap_to_internal(aid, self.db())
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(aid.to_string()))?;
		let internal_oid = if let Some(o) = oid {
			Some(
				crate::model::object::Entity::ap_to_internal(o, self.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(o.to_string()))?
			)
		} else { None };
		self.address_to(Some(internal_aid), internal_oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}

}
