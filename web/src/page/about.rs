use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
	// TODO this is super awful but I recycled it from previous site which was just plain HTML
	//      should really use leptos stuff and global stylesheet rather than doing this awful mess
	let whatpub_script = r#"
		let pronunciations = [ "micro-pub", "mu-pub", "mju-pub", "my-pub", "mi-pub", "you-pub", "oo-pub", "ew-pub", "10e-6-pub", ".000001-pub", "u-pub", "uh-pub", "hu-pub", "huh-pub" ];
		function whatpub() {
			let whatpub = document.getElementById("whatpub");
			whatpub.innerHTML = '/' + pronunciations[Math.floor(Math.random()*pronunciations.length)] + '/';
		}
		setInterval(whatpub, 90000);
		whatpub();
	"#;
	let style_raw = r#"
		.upub-comparison .col-main {
			width: 50%;
		}
		.upub-comparison .col-side {
			width: 50%;
		}
		@media screen and (max-width: 786px) {
			.upub-comparison .col-main {
				width: 100%;
			}
			.upub-comparison .col-side {
				width: 100%;
				margin-bottom: 2em;
			}
		}
		h1.upub-about-title {
			margin-top: 1em;
		}
	"#;
	view! {
		<div>
			<style inner_html=style_raw />
			<div class="mb-3 ml-3">
				<h1 class="upub-about-title">"μpub "<sup><small id="whatpub">"/micro-pub/"</small></sup></h1>
				<blockquote>
					<p>micro social network, federated</p>
				</blockquote>
			</div>
			<p><code>μpub</code>" is a new from-scratch "<a href="https://www.w3.org/TR/activitypub/" target="_blank">ActivityPub</a>" server implementation"</p>
			<p><i>"\"the "<a href="https://en.wikipedia.org/wiki/Fediverse" target="_blank">fediverse</a>" is an ensemble of social networks, which, while independently hosted, can communicate with each other\""</i></p>
			<p>"keep in touch with friends and up to date with content or news with "<code>μpub</code>", unopinionated and intercompatible AP software"</p>
			<hr class="color mt-3" />
			<div class="two-col upub-comparison">
				<div class="col-main">
					<h2 class="mt-1">pros</h2>
					<ul class="plus">
						<li><b>private</b>": object and activity addressing is stored and fully respected for each object and activity, media proxy active by default"</li>
						<li><b>compatible</b>": flawlessly federates with multiple fediverse sources (mastodon, lemmy, pixelfed, pleroma, wordpress, peertube)"</li>
						<li><b>featured</b>": "<code>μpub</code>" boasts both known features (quote posts, media proxy, AP explorer) and new experimental ideas ("<b>"on-demand thread fetching"</b>", granular activity privacy, actor liked feeds)"</li>
						<li><b>simple</b>": just compile the main binary and run it! works right away with sqlite"</li>
						<li><b>fast</b>": both frontend and backend are built leveraging async rust"</li>
						<li><b>flexible</b>": wether you prefer a monolithic simple instance or a distributed high perforance deployment, "<code>μpub</code>" can be set up that way"</li>
					</ul>
				</div>
				<div class="col-side">
					<h2 class="mt-1">cons</h2>
					<ul>
						<li><b>unfinished</b>": "<u>"this project is still under development"</u>"! notable missing features are "<b>media uploads</b>", bookmarks, lists, edit UI and "<b>button undos</b></li>
						<li><b>technical</b>": "<code>μpub</code>" uses many activitypub-native terminology and directly exposes protocol concepts, meaning it may be less intuitive for new users"</li>
						<li><b>moderation</b>": there are no moderation tools available as of now, admins will need to carry out most tasks directly interacting with the database"</li>
						<li><b>zealous</b>": ActivityPub concepts are respected "<span class="moreinfo" title="for example, likes without public addressing won't be shown">as closely as possible</span>", which may lead to small differences from other software's behaviour"</li>
					</ul>
				</div>
			</div>
			<hr class="color mb-3" />
			<p class="center"><small>"this is a preview, follow development on "<a href="https://github.com/alemidev/upub">github</a></small></p>
			<script inner_html=whatpub_script />
		</div>
	}
}
