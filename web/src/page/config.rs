use leptos::*;
use crate::{prelude::*, DEFAULT_COLOR};

#[component]
pub fn ConfigPage(setter: WriteSignal<crate::Config>) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let (color, set_color) = leptos_use::use_css_var("--accent");
	let (_color_rgb, set_color_rgb) = leptos_use::use_css_var("--accent-rgb");

	let previous_color = config.get().accent_color;
	set_color_rgb.set(parse_hex(&previous_color));
	set_color.set(previous_color);

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
			<p>
				<span title="new posts will be fetched automatically when scrolling down enough">
					<input type="checkbox" class="mr-1"
						prop:checked=get_cfg!(infinite_scroll)
						on:input=set_cfg!(infinite_scroll)
					/> infinite scroll
				</span>
			</p>
			<p>
				accent color
				<input type="text" class="ma-1"
					style="width: 8ch;"
					placeholder=DEFAULT_COLOR
					value=color
					on:input=move|ev| {
						let mut val = event_target_value(&ev);
						if val.is_empty() { val = DEFAULT_COLOR.to_string() };
						let mut mock = config.get();
						mock.accent_color = val.clone();
						setter.set(mock);
						set_color_rgb.set(parse_hex(&val));
						set_color.set(val);
				} />
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
			<p class="center"><a href="/web/config/dev" title="access the devtools page">devtools</a></p>
		</div>
	}
}

fn parse_hex(hex: &str) -> String {
	if hex.len() < 7 { return "0, 0, 0".into(); }
	let r = i64::from_str_radix(&hex[1..3], 16).unwrap_or_default();
	let g = i64::from_str_radix(&hex[3..5], 16).unwrap_or_default();
	let b = i64::from_str_radix(&hex[5..7], 16).unwrap_or_default();
	format!("{r}, {g}, {b}")
}
