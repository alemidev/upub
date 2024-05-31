use sea_orm::{ActiveValue::{Set, NotSet}, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use upub::server::addresser::Addresser;

pub async fn relay(ctx: upub::Context, actor: String, accept: bool) -> upub::Result<()> {
	let aid = ctx.aid(&uuid::Uuid::new_v4().to_string());

	let mut activity_model = upub::model::activity::ActiveModel {
		internal: NotSet,
		id: Set(aid.clone()),
		activity_type: Set(apb::ActivityType::Follow),
		actor: Set(ctx.base().to_string()),
		object: Set(Some(actor.clone())),
		target: Set(None),
		published: Set(chrono::Utc::now()),
		to: Set(upub::model::Audience(vec![actor.clone()])),
		bto: Set(upub::model::Audience::default()),
		cc: Set(upub::model::Audience(vec![apb::target::PUBLIC.to_string()])),
		bcc: Set(upub::model::Audience::default()),
	};

	if accept {
		let follow_req = upub::model::activity::Entity::find()
			.filter(upub::model::activity::Column::ActivityType.eq("Follow"))
			.filter(upub::model::activity::Column::Actor.eq(&actor))
			.filter(upub::model::activity::Column::Object.eq(ctx.base()))
			.order_by_desc(upub::model::activity::Column::Published)
			.one(ctx.db())
			.await?
			.expect("no follow request to accept");
		activity_model.activity_type = Set(apb::ActivityType::Accept(apb::AcceptType::Accept));
		activity_model.object = Set(Some(follow_req.id));
	};

	upub::model::activity::Entity::insert(activity_model)
		.exec(ctx.db()).await?;

	ctx.dispatch(ctx.base(), vec![actor, apb::target::PUBLIC.to_string()], &aid, None).await?;

	Ok(())
}
