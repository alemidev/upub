use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns, TransactionTrait, Condition};


pub async fn prune_database(
	ctx: upub::Context,
	days: i64,
	objects: bool,
	activities: bool,
	addressing: bool,
	for_real: bool,
) -> Result<(), sea_orm::DbErr> {

	let limit = chrono::Utc::now() - chrono::TimeDelta::days(days);
	tracing::info!("deleting rows older than {limit}");

	let tx = ctx.db().begin().await?;

	let local_actor_ids = upub::model::actor::Entity::find()
		.filter(upub::model::actor::Column::Domain.eq(ctx.domain()))
		.select_only()
		.select_column(upub::model::actor::Column::Id)
		.into_tuple::<String>()
		.all(&tx)
		.await?;

	let local_actor_internals = upub::model::actor::Entity::find()
		.filter(upub::model::actor::Column::Domain.eq(ctx.domain()))
		.select_only()
		.select_column(upub::model::actor::Column::Internal)
		.into_tuple::<i64>()
		.all(&tx)
		.await?;

	tracing::debug!("local actor ids: {local_actor_ids:?}");

	if objects {
		let res = upub::model::object::Entity::delete_many()
			.filter(upub::model::object::Column::Published.lt(limit))
			.filter(upub::model::object::Column::AttributedTo.is_not_in(&local_actor_ids))
			.filter(
				Condition::any()
					.add(upub::model::object::Column::Audience.is_null())
					.add(upub::model::object::Column::Audience.is_not_in(&local_actor_ids))
			)
			.exec(&tx)
			.await?;
		tracing::info!("deleted {} objects", res.rows_affected);
	}

	if activities {
		let res = upub::model::activity::Entity::delete_many()
			.filter(upub::model::activity::Column::Published.lt(limit))
			.filter(upub::model::activity::Column::Actor.is_not_in(&local_actor_ids))
			.exec(&tx)
			.await?;
		tracing::info!("deleted {} activities", res.rows_affected);
	} 

	if addressing {
		let res = upub::model::addressing::Entity::delete_many()
			.filter(upub::model::addressing::Column::Published.lt(limit))
			.filter(upub::model::addressing::Column::Actor.is_not_in(local_actor_internals))
			.exec(&tx)
			.await?;
		tracing::info!("deleted {} addressings", res.rows_affected);
	}

	if for_real {
		tx.commit().await?;
		tracing::info!("changes committed");
	} else {
		tx.rollback().await?;
		tracing::warn!("this was a dry run! if you really want to proceed, re-run with --for-real");
	}

	Ok(())
}
