use sea_orm::{EntityTrait, TransactionTrait};
use upub::traits::{fetch::{Fetchable, RequestError}, Addresser, Fetcher, Normalizer};

pub async fn fetch(ctx: upub::Context, uri: String, save: bool, actor: Option<String>) -> Result<(), RequestError> {
	use apb::Base;

	let mut pkey = ctx.pkey().to_string();
	let mut from = ctx.base().to_string();
	
	if let Some(actor) = actor {
		let actor_model = upub::model::actor::Entity::find_by_ap_id(&actor)
			.one(ctx.db())
			.await?
			.ok_or_else(|| sea_orm::DbErr::RecordNotFound(actor.clone()))?;

		match actor_model.private_key {
			None => tracing::error!("requested actor lacks a private key, fetching with server key instead"),
			Some(x) => {
				pkey = x;
				from = actor.to_string();
			},
		}
	}

	let mut node = apb::Node::link(uri.to_string());
	if let apb::Node::Link(ref uri) = node {
		if let Ok(href) = uri.href() {
			node = upub::Context::request(reqwest::Method::GET, href, None, &from, &pkey, ctx.domain())
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}
	}


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
