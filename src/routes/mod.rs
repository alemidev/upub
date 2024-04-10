pub mod activitypub;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mastodon")]
pub mod mastodon;

#[cfg(not(feature = "mastodon"))]
pub mod mastodon {
	pub trait MastodonRouter {
		fn mastodon_routes(self) -> Self { self }
	}
	
	impl MastodonRouter for axum::Router<crate::server::Context> {}
}
