use sea_orm::{EntityTrait, IntoActiveModel};

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
		let key = openssl::rsa::Rsa::generate(2048).unwrap();
		let ap_id = self.uid(username.clone());
		let db = self.db();
		let domain = self.domain().to_string();
		let user_model = crate::model::user::Model {
			id: ap_id.clone(),
			name: display_name,
			domain, summary,
			preferred_username: username.clone(),
			following: None,
			following_count: 0,
			followers: None,
			followers_count: 0,
			statuses_count: 0,
			icon: avatar_url,
			image: banner_url,
			inbox: None,
			shared_inbox: None,
			outbox: None,
			actor_type: apb::ActorType::Person,
			created: chrono::Utc::now(),
			updated: chrono::Utc::now(),
			private_key: Some(std::str::from_utf8(&key.private_key_to_pem().unwrap()).unwrap().to_string()),
			public_key: std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap().to_string(),
		};

		crate::model::user::Entity::insert(user_model.into_active_model())
			.exec(db)
			.await?;

		let config_model = crate::model::config::Model {
			id: ap_id.clone(),
			accept_follow_requests: true,
			show_followers_count: true,
			show_following_count: true,
			show_followers: false,
			show_following: false,
		};

		crate::model::config::Entity::insert(config_model.into_active_model())
			.exec(db)
			.await?;

		let credentials_model = crate::model::credential::Model {
			id: ap_id,
			email: username,
			password,
		};

		crate::model::credential::Entity::insert(credentials_model.into_active_model())
			.exec(db)
			.await?;
		
		Ok(())
	}
}
