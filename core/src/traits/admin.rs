use sea_orm::{ActiveValue::{NotSet, Set}, DbErr, EntityTrait};

use crate::ext::JsonVec;

#[allow(async_fn_in_trait)]
pub trait Administrable {
	async fn register_user(
		&self,
		username: String,
		password: String,
		display_name: Option<String>,
		summary: Option<String>,
		avatar_url: Option<String>,
		banner_url: Option<String>,
	) -> Result<(), DbErr>;
}

impl Administrable for crate::Context {
	async fn register_user(
		&self,
		username: String,
		password: String,
		display_name: Option<String>,
		summary: Option<String>,
		avatar_url: Option<String>,
		banner_url: Option<String>,
	) -> Result<(), DbErr> {
		let key = openssl::rsa::Rsa::generate(2048).unwrap();
		let ap_id = self.uid(&username);
		let db = self.db();
		let domain = self.domain().to_string();
		let user_model = crate::model::actor::ActiveModel {
			internal: NotSet,
			id: Set(ap_id.clone()),
			name: Set(display_name),
			domain: Set(domain),
			summary: Set(summary),
			preferred_username: Set(username.clone()),
			fields: Set(JsonVec::default()),
			following: Set(None),
			following_count: Set(0),
			followers: Set(None),
			followers_count: Set(0),
			statuses_count: Set(0),
			also_known_as: Set(JsonVec::default()),
			moved_to: Set(None),
			icon: Set(avatar_url),
			image: Set(banner_url),
			inbox: Set(None),
			shared_inbox: Set(None),
			outbox: Set(None),
			actor_type: Set(apb::ActorType::Person),
			published: Set(chrono::Utc::now()),
			updated: Set(chrono::Utc::now()),
			private_key: Set(Some(std::str::from_utf8(&key.private_key_to_pem().unwrap()).unwrap().to_string())),
			public_key: Set(std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap().to_string()),
		};

		crate::model::actor::Entity::insert(user_model)
			.exec(db)
			.await?;

		let config_model = crate::model::config::ActiveModel {
			internal: NotSet,
			actor: Set(ap_id.clone()),
			accept_follow_requests: Set(true),
			show_followers_count: Set(true),
			show_following_count: Set(true),
			show_followers: Set(false),
			show_following: Set(false),
			show_liked_objects: Set(false),
		};

		crate::model::config::Entity::insert(config_model)
			.exec(db)
			.await?;

		let credentials_model = crate::model::credential::ActiveModel {
			internal: NotSet,
			actor: Set(ap_id),
			login: Set(username),
			password: Set(sha256::digest(password)),
			active: Set(!self.cfg().security.require_user_approval),
		};

		crate::model::credential::Entity::insert(credentials_model)
			.exec(db)
			.await?;
		
		Ok(())
	}
}
