use openssl::rsa::Rsa;
use sea_orm::{EntityTrait, IntoActiveModel};

pub async fn register(
	db: sea_orm::DatabaseConnection,
	domain: String,
) -> crate::Result<()> {
	let key = Rsa::generate(2048).unwrap();
	let test_user = crate::model::user::Model {
		id: format!("{domain}/users/test"),
		name: Some("μpub".into()),
		domain: clean_domain(&domain),
		preferred_username: "test".to_string(),
		summary: Some("hello world! i'm manually generated but served dynamically from db! check progress at https://git.alemi.dev/upub.git".to_string()),
		following: None,
		following_count: 0,
		followers: None,
		followers_count: 0,
		statuses_count: 0,
		icon: Some("https://cdn.alemi.dev/social/circle-square.png".to_string()),
		image: Some("https://cdn.alemi.dev/social/someriver-xs.jpg".to_string()),
		inbox: None,
		shared_inbox: None,
		outbox: None,
		actor_type: apb::ActorType::Person,
		created: chrono::Utc::now(),
		updated: chrono::Utc::now(),
		private_key: Some(std::str::from_utf8(&key.private_key_to_pem().unwrap()).unwrap().to_string()),
		// TODO generate a fresh one every time
		public_key: std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap().to_string(),
	};

	crate::model::user::Entity::insert(test_user.clone().into_active_model()).exec(&db).await?;

	Ok(())
}

// TODO duplicated, make an util?? idk
fn clean_domain(domain: &str) -> String {
	domain
		.replace("http://", "")
		.replace("https://", "")
		.replace('/', "")
}
