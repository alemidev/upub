use apb::{Base, Object};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, EntityTrait, QueryFilter, ColumnTrait};
use upub::{ext::AnyQuery, traits::{fetch::RequestError, Fetcher}};


pub async fn fetch_custom_emojis(ctx: upub::Context, document: String) -> Result<(), RequestError> {
	let document = ctx.pull(&document).await?.document();

	let mut count = 0;
	for tag in document.tag().flat() {
		count += 1;
		if let Ok(doc) = tag.into_inner() {
			use apb::Link;
			if matches!(doc.link_type(), Ok(apb::LinkType::Emoji)) {
				let name = apb::Link::name(&doc).unwrap_or_default().replace(':', "");
				let domain = upub::Context::server(&document.id().unwrap_or_default());
				let uri = doc.icon().into_inner().and_then(|x| x.url().id()).unwrap_or_default();
				if !name.is_empty()
					&& !domain.is_empty()
					&& !uri.is_empty()
					&& !upub::model::emoji::Entity::find()
						.filter(upub::model::emoji::Column::Name.eq(&name))
						.filter(upub::model::emoji::Column::Domain.eq(&domain))
						.any(ctx.db())
						.await?
					// TODO every time we resolve an user we make multiple queries
				{
					tracing::info!("adding {name}:{domain} -> {uri}");
					upub::model::emoji::ActiveModel {
						internal: NotSet,
						domain: Set(domain),
						name: Set(name),
						uri: Set(uri),
					}
						.insert(ctx.db())
						.await?;
				} else {
					tracing::warn!("{name}:{domain} -> {uri} already exists");
				}
			} else {
				tracing::warn!("shipping non-emoji tag");
			}
		} else {
			tracing::warn!("could not extract tag data from document");
		}
	}

	if count == 0 {
		tracing::warn!("no tags in this document");
	}

	Ok(())
}
