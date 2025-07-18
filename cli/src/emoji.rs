use upub::traits::{fetch::RequestError, Fetcher, Normalizer};

pub async fn fetch_custom_emojis(ctx: upub::Context, document: String) -> Result<(), RequestError> {
	let document = ctx.pull(&document).await?.document();

	ctx.insert_tags(document, ctx.db(), 0, false, false, true).await?;

	Ok(())
}
