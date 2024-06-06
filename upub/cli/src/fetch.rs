use sea_orm::EntityTrait;
use upub::traits::{fetch::{Fetchable, PullError}, Normalizer};

pub async fn fetch(ctx: upub::Context, uri: String, save: bool) -> Result<(), PullError> {
	use apb::Base;

	let mut node = apb::Node::link(uri.to_string());
	node.fetch(&ctx).await?;

	let obj = node.extract().expect("node still empty after fetch?");

	println!("{}", serde_json::to_string_pretty(&obj).unwrap());

	if save {
		match obj.base_type() {
			Ok(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				upub::model::actor::Entity::insert(
					upub::AP::actor_q(&obj).unwrap()
				).exec(ctx.db()).await.unwrap();
			},
			Ok(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				ctx.insert_activity(obj).await.unwrap();
			},
			Ok(apb::BaseType::Object(apb::ObjectType::Note)) => {
				ctx.insert_object(obj).await.unwrap();
			},
			Ok(apb::BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Ok(apb::BaseType::Link(_)) => tracing::error!("fetched another link?"),
			Err(_) => tracing::error!("no type on object"),
		}
	}

	Ok(())
}
