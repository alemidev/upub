use sea_orm::{EntityTrait, TransactionTrait};
use upub::traits::{fetch::{Fetchable, PullError}, Addresser, Normalizer};

pub async fn fetch(ctx: upub::Context, uri: String, save: bool) -> Result<(), PullError> {
	use apb::Base;

	let mut node = apb::Node::link(uri.to_string());
	node.fetch(&ctx).await?;


	let obj = node.extract().expect("node still empty after fetch?");

	println!("{}", serde_json::to_string_pretty(&obj).unwrap());

	if save {
		let tx = ctx.db().begin().await?;
		match obj.base_type() {
			Ok(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				upub::model::actor::Entity::insert(upub::AP::actor_q(&obj, None)?)
					.exec(&tx)
					.await?;
			},
			Ok(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				let act = ctx.insert_activity(obj, &tx).await?;
				ctx.address((Some(&act), None), &tx).await?;
			},
			Ok(apb::BaseType::Object(apb::ObjectType::Note)) => {
				let obj = ctx.insert_object(obj, &tx).await?;
				ctx.address((None, Some(&obj)), &tx).await?;
			},
			Ok(apb::BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Ok(apb::BaseType::Link(_)) => tracing::error!("fetched another link?"),
			Err(_) => tracing::error!("no type on object"),
		}
		tx.commit().await?;
	}

	Ok(())
}
