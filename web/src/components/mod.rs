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
			let delta = chrono::Utc::now() - t;
			let pretty = if delta.num_seconds() < 60 {
				format!("{}s ago", delta.num_seconds())
			} else if delta.num_minutes() < 60 {
				format!("{}m ago", delta.num_minutes())
			} else if delta.num_hours() < 24 {
				format!("{}h ago", delta.num_hours())
			} else if delta.num_days() < 90 {
				format!("{}d ago", delta.num_days())
			} else {
				t.format("%d/%m/%Y").to_string()
			};
			let rfc = t.to_rfc2822();
			Some(view! {
				<small title={rfc}>{pretty}</small>
			})
		},
		None => None,
	}
}

pub const PRIVACY_PUBLIC : &str = "🪩";
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
		<span class="emoji ml-1 mr-s moreinfo" title={audience} >{privacy}</span>
	}
}
