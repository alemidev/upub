use openssl::rsa::Rsa;
use sea_orm::{ActiveValue::{NotSet, Set}, DatabaseConnection, EntityTrait};

use crate::model;

pub async fn application(
	domain: String,
	base_url: String,
	db: &DatabaseConnection
) -> crate::Result<(model::actor::Model, model::instance::Model)> {
	Ok((
		match model::actor::Entity::find_by_ap_id(&base_url).one(db).await? {
			Some(model) => model,
			None => {
				tracing::info!("generating application keys");
				let rsa = Rsa::generate(2048)?;
				let privk = std::str::from_utf8(&rsa.private_key_to_pem()?)?.to_string();
				let pubk = std::str::from_utf8(&rsa.public_key_to_pem()?)?.to_string();
				let system = model::actor::ActiveModel {
					internal: NotSet,
					id: Set(base_url.clone()),
					domain: Set(domain.clone()),
					preferred_username: Set(domain.clone()),
					actor_type: Set(apb::ActorType::Application),
					private_key: Set(Some(privk)),
					public_key: Set(pubk),
					following: Set(None),
					following_count: Set(0),
					followers: Set(None),
					followers_count: Set(0),
					statuses_count: Set(0),
					summary: Set(Some("micro social network, federated".to_string())),
					name: Set(Some("μpub".to_string())),
					image: Set(None),
					icon: Set(Some("https://cdn.alemi.dev/social/circle-square.png".to_string())),
					inbox: Set(Some(format!("{base_url}/inbox"))),
					shared_inbox: Set(Some(format!("{base_url}/inbox"))),
					outbox: Set(Some(format!("{base_url}/outbox"))),
					published: Set(chrono::Utc::now()),
					updated: Set(chrono::Utc::now()),
				};
				model::actor::Entity::insert(system).exec(db).await?;
				// sqlite doesn't resurn last inserted id so we're better off just querying again, it's just one time
				model::actor::Entity::find().one(db).await?.expect("could not find app actor just inserted")
			}
		},

		match model::instance::Entity::find_by_domain(&domain).one(db).await? {
			Some(model) => model,
			None => {
				tracing::info!("generating instance counters");
				let system = model::instance::ActiveModel {
					internal: NotSet,
					domain: Set(domain.clone()),
					down_since: Set(None),
					software: Set(Some("upub".to_string())),
					version: Set(Some(crate::VERSION.to_string())),
					name: Set(None),
					icon: Set(None),
					users: Set(Some(0)),
					posts: Set(Some(0)),
					published: Set(chrono::Utc::now()),
					updated: Set(chrono::Utc::now()),
				};
				model::instance::Entity::insert(system).exec(db).await?;
				// sqlite doesn't resurn last inserted id so we're better off just querying again, it's just one time
				model::instance::Entity::find().one(db).await?.expect("could not find app instance just inserted")
			}
		}
	))
}
