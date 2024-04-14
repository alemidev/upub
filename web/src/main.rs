use leptos::*;

use leptos_use::{use_cookie, utils::FromToStringCodec};
use upub_web::{
	LoginBox, PostBox, TimelinePicker, Timeline
};

fn main() {
	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();
	let (cookie, set_cookie) = use_cookie::<String, FromToStringCodec>("token");
	let (timeline, set_timeline) = create_signal("https://feditest.alemi.dev/users/test/inbox/page".to_string());
	mount_to_body(
		move || view! {
			<nav class="w-100">
				<p>
					<code>μpub</code>
					<small class="ml-1 mr-1" ><a class="clean" href="/" >micro social network, federated</a></small>
					/* TODO kinda jank with the float but whatever, will do for now */
					<small style="float: right" ><a href="https://git.alemi.dev/upub.git" >src</a></small>
				</p>
			</nav>
			<hr />
			<div class="container">
				<div class="two-col">
					<div class="col-side">
						<LoginBox
							tx=set_cookie
							rx=cookie
						/>
						<hr />
						<PostBox token=cookie />
						<hr />
					</div>
					<div class="col-main">
						<TimelinePicker tx=set_timeline rx=timeline />
						<Timeline
							feed=timeline
							token=cookie
						/>
					</div>
				</div>
			</div>
		}
	);
}
