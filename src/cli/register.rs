use openssl::rsa::Rsa;
use sea_orm::{EntityTrait, IntoActiveModel};

pub async fn register(
	ctx: crate::server::Context,
	username: String,
	password: String,
	display_name: Option<String>,
	summary: Option<String>,
	avatar_url: Option<String>,
	banner_url: Option<String>,
) -> crate::Result<()> {
	ctx.register_user(
		username.clone(),
		password,
		display_name,
		summary,
		avatar_url,
		banner_url,
	).await?;

	tracing::info!("registered new user: {username}");
}

// TODO duplicated, make an util?? idk
fn clean_domain(domain: &str) -> String {
	domain
		.replace("http://", "")
		.replace("https://", "")
		.replace('/', "")
}
