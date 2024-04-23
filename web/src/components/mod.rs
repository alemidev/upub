mod activity;
pub use activity::ActivityLine;

mod object;
pub use object::Object;

mod user;
pub use user::ActorBanner;

mod timeline;
pub use timeline::{TimelineFeed, TimelineReplies, Timeline};

use leptos::*;

#[component]
pub fn DateTime(t: Option<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
	match t {
		Some(t) => {
			let pretty = t.format("%Y/%m/%d %H:%M:%S").to_string();
			let rfc = t.to_rfc3339();
			Some(view! {
				<small title={rfc}>{pretty}</small>
			})
		},
		None => None,
	}
}

pub const PRIVACY_PUBLIC : &str = "💿";
pub const PRIVACY_FOLLOWERS : &str = "🔒";
pub const PRIVACY_PRIVATE : &str = "📨";

#[component]
pub fn PrivacyMarker(addressed: Vec<String>) -> impl IntoView {
	let privacy = if addressed.iter().any(|x| x == apb::target::PUBLIC) {
		PRIVACY_PUBLIC
	} else if addressed.iter().any(|x| x.ends_with("/followers")) {
		PRIVACY_FOLLOWERS
	} else {
		PRIVACY_PRIVATE
	};
	let audience = format!("[ {} ]", addressed.join(", "));
	view! {
		<span class="emoji ml-1 moreinfo" title={audience} >{privacy}</span>
	}
}
