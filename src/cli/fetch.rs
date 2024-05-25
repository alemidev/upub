use sea_orm::EntityTrait;

use crate::server::fetcher::Fetchable;

pub async fn fetch(ctx: crate::server::Context, uri: String, save: bool) -> crate::Result<()> {
	use apb::Base;

	let mut node = apb::Node::link(uri.to_string());
	node.fetch(&ctx).await?;

	let obj = node.get().expect("node still empty after fetch?");

	if save {
		match obj.base_type() {
			Some(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				crate::model::actor::Entity::insert(
					crate::model::actor::ActiveModel::new(obj).unwrap()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				crate::model::activity::Entity::insert(
					crate::model::activity::ActiveModel::new(obj).unwrap()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Note)) => {
				crate::model::object::Entity::insert(
					crate::model::object::ActiveModel::new(obj).unwrap()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Some(apb::BaseType::Link(_)) => tracing::error!("fetched another link?"),
			None => tracing::error!("no type on object"),
		}
	}

	println!("{}", serde_json::to_string_pretty(&obj).unwrap());

	Ok(())
}
