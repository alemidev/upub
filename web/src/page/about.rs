use leptos::*;
use crate::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>about</Breadcrumb>
			<div class="mt-s mb-s" >
				<p><code>μpub</code>" is a micro social network powered by "<a href="">ActivityPub</a></p>
				<p><i>"the "<a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>" is an ensemble of social networks, which, while independently hosted, can communicate with each other"</i></p>
				<p>content is aggregated in timelines, logged out users can only access the global server timeline</p>
				<hr />
				<p>"while somewhat usable, "<code>μpub</code>" is under active development and still lacks some mainstream features (such as hashtags or lists)"</p>
				<p>"if you would like to contribute to "<code>μpub</code>"'s development, get in touch and check "<a href="https://github.com/alemidev/upub" target="_blank">github</a>" or "<a href="https://moonlit.technology/alemi/upub.git" target="_blank">forgejo</a></p>
			</div>
		</div>
	}
}
