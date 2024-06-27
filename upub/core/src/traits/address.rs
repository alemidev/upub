use apb::target::Addressed;
use sea_orm::{ActiveValue::{NotSet, Set}, ConnectionTrait, DbErr, EntityTrait, QuerySelect, SelectColumns};

use crate::traits::fetch::Fetcher;

#[async_trait::async_trait]
pub trait Addresser {
	async fn deliver(&self, to: Vec<String>, aid: &str, from: &str, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
	async fn address_object(&self, object: &crate::model::object::Model, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
	async fn address_activity(&self, activity: &crate::model::activity::Model, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
}

#[async_trait::async_trait]
impl Addresser for crate::Context {
	async fn deliver(&self, to: Vec<String>, aid: &str, from: &str, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
		let to = expand_addressing(to, tx).await?;
		let mut deliveries = Vec::new();
		for target in to.into_iter()
			.filter(|to| !to.is_empty())
			.filter(|to| crate::Context::server(to) != self.domain())
			.filter(|to| to != apb::target::PUBLIC)
		{
			// TODO fetch concurrently
			match self.fetch_user(&target, tx).await {
				Ok(crate::model::actor::Model { inbox: Some(inbox), .. }) => deliveries.push(
					crate::model::job::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						actor: Set(from.to_string()),
						job_type: Set(crate::model::job::JobType::Delivery),
						payload: Set(None),
						// TODO we should resolve each user by id and check its inbox because we can't assume
						// it's /actors/{id}/inbox for every software, but oh well it's waaaaay easier now
						target: Set(Some(inbox)),
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
			crate::model::job::Entity::insert_many(deliveries)
				.exec(tx)
				.await?;
		}

		// TODO can we make deliveries instant? for better UX
		// self.dispatcher().wakeup();

		Ok(())
	}

	async fn address_object(&self, object: &crate::model::object::Model, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
		let to = expand_addressing(object.addressed(), tx).await?;
		address_to(self, to, None, Some(object.internal), self.is_local(&object.id), tx).await
	}

	async fn address_activity(&self, activity: &crate::model::activity::Model, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
		let to = expand_addressing(activity.addressed(), tx).await?;
		address_to(self, to, Some(activity.internal), None, self.is_local(&activity.id), tx).await
	}
}

async fn address_to(ctx: &crate::Context, to: Vec<String>, aid: Option<i64>, oid: Option<i64>, local: bool, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
	// TODO address_to became kind of expensive, with these two selects right away and then another
	//      select for each target we're addressing to... can this be improved??
	let now = chrono::Utc::now();
	let mut addressing = Vec::new();
	for target in to.into_iter()
		.filter(|to| !to.is_empty())
		.filter(|to| !to.ends_with("/followers"))
		.filter(|to| local || to.as_str() == apb::target::PUBLIC || ctx.is_local(to))
	{
		let (server, actor) = if target == apb::target::PUBLIC { (None, None) } else {
			match (
				crate::model::instance::Entity::domain_to_internal(&crate::Context::server(&target), tx).await?,
				crate::model::actor::Entity::ap_to_internal(&target, tx).await?,
			) {
				(Some(server), Some(actor)) => (Some(server), Some(actor)),
				(None, _) => { tracing::error!("failed resolving domain of {target}"); continue; },
				(_, None) => { tracing::error!("failed resolving actor {target}"); continue; },
			}
		};
		addressing.push(
			crate::model::addressing::ActiveModel {
				internal: NotSet,
				instance: Set(server),
				actor: Set(actor),
				activity: Set(aid),
				object: Set(oid),
				published: Set(now),
			}
		);
	}

	if !addressing.is_empty() {
		crate::model::addressing::Entity::insert_many(addressing)
			.exec(tx)
			.await?;
	}

	Ok(())
}

async fn expand_addressing(targets: Vec<String>, tx: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
	let mut out = Vec::new();
	for target in targets {
		// TODO this is definitely NOT a reliable way to expand followers collections...
		//      we should add an index on following field in users and try to search for that: no
		//      guarantee that all followers collections end with 'followers'! once we get the actual
		//      user we can resolve their followers with the relations table
		//   !  NOTE THAT local users have followers set to NULL, either fill all local users followers
		//      field or manually check if it's local and then do the .ends_with("/followers")
		// TODO should also expand /following
		// TODO should probably expand audience too but it's not reachable anymore from here, should we
		//      count audience field too in the .addressed() trait? maybe pre-expand it because it's
		//      only used for groups anyway??
		if target.ends_with("/followers") {
			let target_id = target.replace("/followers", "");
			let target_internal = crate::model::actor::Entity::ap_to_internal(&target_id, tx)
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(target_id.clone()))?;
			let mut followers = crate::Query::related(None, Some(target_internal), false)
				.select_only()
				.select_column(crate::model::actor::Column::Id)
				.into_tuple::<String>()
				.all(tx)
				.await?;
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
