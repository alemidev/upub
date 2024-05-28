use sea_orm::{ActiveValue::{NotSet, Set}, EntityTrait};

use crate::model;

use super::{fetcher::Fetcher, Context};


#[axum::async_trait]
pub trait Addresser {
	async fn expand_addressing(&self, targets: Vec<String>) -> crate::Result<Vec<String>>;
	async fn address_to(&self, aid: Option<i64>, oid: Option<i64>, targets: &[String]) -> crate::Result<()>;
	async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> crate::Result<()>;
	//#[deprecated = "should probably directly invoke address_to() since we most likely have internal ids at this point"]
	async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()>;
}

#[axum::async_trait]
impl Addresser for super::Context {
	async fn expand_addressing(&self, targets: Vec<String>) -> crate::Result<Vec<String>> {
		let mut out = Vec::new();
		for target in targets {
			if target.ends_with("/followers") {
				let target_id = target.replace("/followers", "");
				let mut followers = model::relation::Entity::followers(&target_id, self.db()).await?;
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

	async fn address_to(&self, aid: Option<i64>, oid: Option<i64>, targets: &[String]) -> crate::Result<()> {
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
					model::instance::Entity::domain_to_internal(&Context::server(target), self.db()).await,
					model::actor::Entity::ap_to_internal(target, self.db()).await,
				) {
					(Ok(server), Ok(actor)) => (Some(server), Some(actor)),
					(Err(e), Ok(_)) => { tracing::error!("failed resolving domain: {e}"); continue; },
					(Ok(_), Err(e)) => { tracing::error!("failed resolving actor: {e}"); continue; },
					(Err(es), Err(ea)) => { tracing::error!("failed resolving domain ({es}) and actor ({ea})"); continue; },
				}
			};
			addressing.push(
				model::addressing::ActiveModel {
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
			model::addressing::Entity::insert_many(addressing)
				.exec(self.db())
				.await?;
		}

		Ok(())
	}

	async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> crate::Result<()> {
		let mut deliveries = Vec::new();
		for target in targets.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| Context::server(to) != self.domain())
			.filter(|to| to != &apb::target::PUBLIC)
		{
			// TODO fetch concurrently
			match self.fetch_user(target).await {
				Ok(model::actor::Model { inbox: Some(inbox), .. }) => deliveries.push(
					model::delivery::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						actor: Set(from.to_string()),
						// TODO we should resolve each user by id and check its inbox because we can't assume
						// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
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
			model::delivery::Entity::insert_many(deliveries)
				.exec(self.db())
				.await?;
		}

		self.dispatcher().wakeup();

		Ok(())
	}

	//#[deprecated = "should probably directly invoke address_to() since we most likely have internal ids at this point"]
	async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()> {
		let addressed = self.expand_addressing(activity_targets).await?;
		let internal_aid = model::activity::Entity::ap_to_internal(aid, self.db()).await?;
		let internal_oid = if let Some(o) = oid { Some(model::object::Entity::ap_to_internal(o, self.db()).await?) } else { None };
		self.address_to(Some(internal_aid), internal_oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}

}
