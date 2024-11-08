use std::collections::BTreeSet;

use apb::target::Addressed;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns};

use crate::traits::fetch::Fetcher;

#[allow(async_fn_in_trait)]
pub trait Addresser {
	async fn deliver(&self, to: Vec<String>, aid: &str, from: &str, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
	async fn address(&self, activity: Option<&crate::model::activity::Model>, object: Option<&crate::model::object::Model>, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
}

impl Addresser for crate::Context {
	async fn deliver(&self, to: Vec<String>, aid: &str, from: &str, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
		let to = expand_addressing(to, None, tx).await?;
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
						error: Set(None),
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

	async fn address(&self, activity: Option<&crate::model::activity::Model>, object: Option<&crate::model::object::Model>, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
		match (activity, object) {
			(None, None) => Ok(()),
			(Some(activity), None) => {
				let to = expand_addressing(activity.addressed(), None, tx).await?;
				address_to(self, to, Some(activity.internal), None, self.is_local(&activity.id), activity.published, tx).await
			},
			(None, Some(object)) => {
				let to = expand_addressing(object.addressed(), object.audience.clone(), tx).await?;
				address_to(self, to, None, Some(object.internal), self.is_local(&object.id), object.published, tx).await
			},
			(Some(activity), Some(object)) => {
				let to_activity = BTreeSet::from_iter(expand_addressing(activity.addressed(), None, tx).await?);
				let to_object = BTreeSet::from_iter(expand_addressing(object.addressed(), object.audience.clone(), tx).await?);
				let to_common = to_activity.intersection(&to_object).cloned().collect();
				address_to(self, to_common, Some(activity.internal), Some(object.internal), self.is_local(&activity.id), activity.published, tx).await?;
				let to_only_activity = (&to_activity - &to_object).into_iter().collect();
				address_to(self, to_only_activity, Some(activity.internal), None, self.is_local(&activity.id), activity.published, tx).await?;
				let to_only_object = (&to_object - &to_activity).into_iter().collect();
				address_to(self, to_only_object, None, Some(object.internal), self.is_local(&activity.id), object.published, tx).await?;
				Ok(())
			},
		}
	}
}

async fn address_to(ctx: &crate::Context, to: Vec<String>, aid: Option<i64>, oid: Option<i64>, local: bool, when: chrono::DateTime<chrono::Utc>, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
	// TODO address_to became kind of expensive, with these two selects right away and then another
	//      select for each target we're addressing to... can this be improved??
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

		// TODO this is yet another select to insert, can we avoid merging these or think of something
		//      else entirely??

		// if we discovered this object previously, merge its addressing with older entry so it doesnt
		// appear twice in timelines
		if let (Some(aid), Some(oid)) = (aid, oid) {
			if let Some(prev) = crate::model::addressing::Entity::find()
				.filter(crate::model::addressing::Column::Activity.is_null())
				.filter(crate::model::addressing::Column::Object.eq(oid))
				.filter(crate::model::addressing::Column::Actor.eq(actor))
				.filter(crate::model::addressing::Column::Instance.eq(server))
				.one(tx)
				.await?
			{
				let mut prev = prev.into_active_model();
				prev.activity = Set(Some(aid));
				prev.object = Set(Some(oid));
				prev.update(tx).await?;
				continue; // no need to insert this one
			}
		}

		addressing.push(
			crate::model::addressing::ActiveModel {
				internal: NotSet,
				instance: Set(server),
				actor: Set(actor),
				activity: Set(aid),
				object: Set(oid),
				published: Set(when),
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

async fn expand_addressing(targets: Vec<String>, audience: Option<String>, tx: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
	let mut out = Vec::new();
	if let Some(audience) = audience {
		if let Some(internal) = crate::model::actor::Entity::ap_to_internal(&audience, tx).await? {
			let mut members = crate::Query::related(None, Some(internal), false)
				.select_only()
				.select_column(crate::model::actor::Column::Id)
				.into_tuple::<String>()
				.all(tx)
				.await?;
			out.append(&mut members);
		}
	}

	for target in targets {
		if let Some(followers_of) = crate::model::actor::Entity::find()
			.filter(crate::model::actor::Column::Followers.eq(&target))
			.select_only()
			.select_column(crate::model::actor::Column::Internal)
			.into_tuple::<i64>()
			.one(tx)
			.await?
		{
			let mut followers = crate::Query::related(None, Some(followers_of), false)
				.select_only()
				.select_column(crate::model::actor::Column::Id)
				.into_tuple::<String>()
				.all(tx)
				.await?;
			out.append(&mut followers);
		} else {
			out.push(target);
		}
	}
	Ok(out)
}
