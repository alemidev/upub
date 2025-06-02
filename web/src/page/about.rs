use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
	view! {
		<div>
			<div class="mt-s mb-s" >
				<p><code>μpub</code>" is a micro social network powered by "<a href="">ActivityPub</a></p>
				<p><i>"the "<a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>" is an ensemble of social networks, which, while independently hosted, can communicate with each other"</i></p>
				<p>content is aggregated in timelines, logged out users can only access the global server timeline</p>
				<hr />
				<p>"more information on "<a href="https://github.com/alemidev/upub" target="_blank">"μpub github page"</a></p>
			</div>
		</div>
	}
}
