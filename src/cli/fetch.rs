use sea_orm::EntityTrait;

use crate::server::{fetcher::Fetchable, normalizer::Normalizer, Context};

pub async fn fetch(ctx: crate::server::Context, uri: String, save: bool) -> crate::Result<()> {
	use apb::Base;

	let mut node = apb::Node::link(uri.to_string());
	node.fetch(&ctx).await?;

	let obj = node.extract().expect("node still empty after fetch?");
	let server = Context::server(&uri);

	println!("{}", serde_json::to_string_pretty(&obj).unwrap());

	if save {
		match obj.base_type() {
			Some(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				crate::model::actor::Entity::insert(
					crate::model::actor::ActiveModel::new(&obj).unwrap()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				ctx.insert_activity(obj, Some(server)).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Note)) => {
				ctx.insert_object(obj, Some(server)).await.unwrap();
			},
			Some(apb::BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Some(apb::BaseType::Link(_)) => tracing::error!("fetched another link?"),
			None => tracing::error!("no type on object"),
		}
	}

	Ok(())
}
