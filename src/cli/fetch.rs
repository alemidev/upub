use sea_orm::{EntityTrait, IntoActiveModel};

use crate::server::fetcher::Fetchable;

pub async fn fetch(db: sea_orm::DatabaseConnection, domain: String, uri: String, save: bool) -> crate::Result<()> {
	use apb::Base;

	let ctx = crate::server::Context::new(db, domain)
		.await.expect("failed creating server context");

	let mut node = apb::Node::link(uri.to_string());
	node.fetch(&ctx).await?;

	let obj = node.get().expect("node still empty after fetch?");

	if save {
		match obj.base_type() {
			Some(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				crate::model::user::Entity::insert(
					crate::model::user::Model::new(obj).unwrap().into_active_model()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				crate::model::activity::Entity::insert(
					crate::model::activity::Model::new(obj).unwrap().into_active_model()
				).exec(ctx.db()).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Note)) => {
				crate::model::object::Entity::insert(
					crate::model::object::Model::new(obj).unwrap().into_active_model()
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
