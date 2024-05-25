use sea_orm::{ActiveValue::{Set, NotSet}, EntityTrait};

use crate::errors::UpubError;

#[axum::async_trait]
pub trait Administrable {
	async fn register_user(
		&self,
		username: String,
		password: String,
		display_name: Option<String>,
		summary: Option<String>,
		avatar_url: Option<String>,
		banner_url: Option<String>,
	) -> crate::Result<()>;
}

#[axum::async_trait]
impl Administrable for super::Context {
	async fn register_user(
		&self,
		username: String,
		password: String,
		display_name: Option<String>,
		summary: Option<String>,
		avatar_url: Option<String>,
		banner_url: Option<String>,
	) -> crate::Result<()> {
		let local_instance = crate::model::instance::Entity::find_by_domain(self.domain())
			.one(self.db())
			.await?
			.ok_or_else(UpubError::internal_server_error)?;
		let key = openssl::rsa::Rsa::generate(2048).unwrap();
		let ap_id = self.uid(&username);
		let db = self.db();
		let domain = self.domain().to_string();
		let user_model = crate::model::actor::ActiveModel {
			id: NotSet,
			ap_id: Set(ap_id.clone()),
			name: Set(display_name),
			instance: Set(local_instance.id),
			summary: Set(summary),
			preferred_username: Set(username.clone()),
			following: Set(None),
			following_count: Set(0),
			followers: Set(None),
			followers_count: Set(0),
			statuses_count: Set(0),
			icon: Set(avatar_url),
			image: Set(banner_url),
			inbox: Set(None),
			shared_inbox: Set(None),
			outbox: Set(None),
			actor_type: Set(apb::ActorType::Person),
			created: Set(chrono::Utc::now()),
			updated: Set(chrono::Utc::now()),
			private_key: Set(Some(std::str::from_utf8(&key.private_key_to_pem().unwrap()).unwrap().to_string())),
			public_key: Set(std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap().to_string()),
		};

		crate::model::actor::Entity::insert(user_model)
			.exec(db)
			.await?;

		let user_model = crate::model::actor::Entity::find_by_ap_id(&ap_id)
			.one(db)
			.await?
			.ok_or_else(UpubError::internal_server_error)?;

		let config_model = crate::model::config::ActiveModel {
			id: NotSet,
			actor: Set(user_model.id),
			accept_follow_requests: Set(true),
			show_followers_count: Set(true),
			show_following_count: Set(true),
			show_followers: Set(false),
			show_following: Set(false),
		};

		crate::model::config::Entity::insert(config_model)
			.exec(db)
			.await?;

		let credentials_model = crate::model::credential::ActiveModel {
			id: NotSet,
			actor: Set(user_model.id),
			login: Set(username),
			password: Set(password),
		};

		crate::model::credential::Entity::insert(credentials_model)
			.exec(db)
			.await?;
		
		Ok(())
	}
}
