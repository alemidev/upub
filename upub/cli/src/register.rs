use upub::server::admin::Administrable;

pub async fn register(
	ctx: upub::Context,
	username: String,
	password: String,
	display_name: Option<String>,
	summary: Option<String>,
	avatar_url: Option<String>,
	banner_url: Option<String>,
) -> upub::Result<()> {
	ctx.register_user(
		username.clone(),
		password,
		display_name,
		summary,
		avatar_url,
		banner_url,
	).await?;

	tracing::info!("registered new user: {username}");
	
	Ok(())
}