mod login;
pub use login::*;

mod navigation;
pub use navigation::*;

mod user;
pub use user::*;

mod post;
pub use post::*;

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

#[component]
pub fn PrivacyMarker<'a>(
	privacy: Privacy,
	#[prop(optional)] to: &'a [String],
	#[prop(optional)] cc: &'a [String],
	#[prop(optional)] big: bool,
) -> impl IntoView {
	let to_txt = if to.is_empty() { String::new() } else { format!("to: {}", to.join(", ")) };
	let cc_txt = if cc.is_empty() { String::new() } else { format!("cc: {}", cc.join(", ")) };
	let audience = format!("{to_txt}\n{cc_txt}");
	view! {
		<span class:big-emoji=big class="emoji ml-1 mr-s moreinfo" title={audience} >{privacy.icon()}</span>
	}
}
