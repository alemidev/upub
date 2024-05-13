use leptos::*;
use crate::prelude::*;

#[component]
pub fn ConfigPage(setter: WriteSignal<crate::Config>) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");

	macro_rules! get_cfg {
		(filter $field:ident) => {
			move || config.get().filters.$field
		};
		($field:ident) => {
			move || config.get().$field
		};
	}

	macro_rules! set_cfg {
		($field:ident) => {
			move |ev| {
				let mut mock = config.get();
				mock.$field = event_target_checked(&ev);
				setter.set(mock);
			}
		};
		(filter $field:ident) => {
			move |ev| {
				let mut mock = config.get();
				mock.filters.$field = event_target_checked(&ev);
				setter.set(mock);
			}
		};
	}

	view! {
		<div>
			<Breadcrumb>config</Breadcrumb>
			<p class="center mt-0"><small>config is saved in your browser local storage</small></p>
			<p>
				<span title="embedded video attachments will loop like gifs if this option is enabled">
					<input type="checkbox" class="mr-1"
						prop:checked=get_cfg!(loop_videos)
						on:input=set_cfg!(loop_videos)
					/> loop videos
				</span>
			</p>
			<p>
				<span title="any post with a summary is considered to have a content warning, and collapsed by default if this option is enabled">
					<input type="checkbox" class="mr-1"
						prop:checked=get_cfg!(collapse_content_warnings)
						on:input=set_cfg!(collapse_content_warnings)
					/> collapse content warnings
				</span>
			</p>
			<hr />
			<p><code title="unchecked elements won't show in timelines">filters</code></p>
			<ul>
					<li><span title="like activities"><input type="checkbox" prop:checked=get_cfg!(filter likes) on:input=set_cfg!(filter likes) />" likes"</span></li>
					<li><span title="create activities with object"><input type="checkbox" prop:checked=get_cfg!(filter creates) on:input=set_cfg!(filter creates)/>" creates"</span></li>
					<li><span title="announce activities with object"><input type="checkbox" prop:checked=get_cfg!(filter announces) on:input=set_cfg!(filter announces) />" announces"</span></li>
					<li><span title="follow, accept and reject activities"><input type="checkbox" prop:checked=get_cfg!(filter follows) on:input=set_cfg!(filter follows) />" follows"</span></li>
					<li><span title="objects without a related activity to display"><input type="checkbox" prop:checked=get_cfg!(filter orphans) on:input=set_cfg!(filter orphans) />" orphans"</span></li>
			</ul>
			<hr />
			<p><a href="/web/config/dev" title="access the devtools page">devtools</a></p>
		</div>
	}
}
