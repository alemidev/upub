use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

pub async fn relay(ctx: crate::server::Context, actor: String, accept: bool) -> crate::Result<()> {
	let aid = ctx.aid(uuid::Uuid::new_v4().to_string());

	let mut activity_model = crate::model::activity::Model {
		id: aid.clone(),
		activity_type: apb::ActivityType::Follow,
		actor: ctx.base(),
		object: Some(actor.clone()),
		target: None,
		published: chrono::Utc::now(),
		to: crate::model::Audience(vec![actor.clone()]),
		bto: crate::model::Audience::default(),
		cc: crate::model::Audience(vec![apb::target::PUBLIC.to_string()]),
		bcc: crate::model::Audience::default(),
	};

	if accept {
		let follow_req = crate::model::activity::Entity::find()
			.filter(crate::model::activity::Column::ActivityType.eq("Follow"))
			.filter(crate::model::activity::Column::Actor.eq(&actor))
			.filter(crate::model::activity::Column::Object.eq(ctx.base()))
			.order_by_desc(crate::model::activity::Column::Published)
			.one(ctx.db())
			.await?
			.expect("no follow request to accept");
		activity_model.activity_type = apb::ActivityType::Accept(apb::AcceptType::Accept);
		activity_model.object = Some(follow_req.id);
	};

	crate::model::activity::Entity::insert(activity_model.into_active_model())
		.exec(ctx.db()).await?;

	ctx.dispatch(&ctx.base(), vec![actor, apb::target::PUBLIC.to_string()], &aid, None).await?;

	Ok(())
}
