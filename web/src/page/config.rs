use apb::{ActivityMut, DocumentMut, Object, ObjectMut};
use leptos::*;
use crate::{prelude::*, DEFAULT_COLOR};

#[component]
pub fn ConfigPage(setter: WriteSignal<crate::Config>) -> impl IntoView {
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let auth = use_context::<Auth>().expect("missing auth context");
	let (color, set_color) = leptos_use::use_css_var("--accent");
	let (_color_rgb, set_color_rgb) = leptos_use::use_css_var("--accent-rgb");

	// TODO should this be responsive? idk
	let previous_color = config.get_untracked().accent_color;
	set_color_rgb.set(parse_hex(&previous_color));
	set_color.set(previous_color);

	let display_name_ref: NodeRef<html::Input> = create_node_ref();
	let summary_ref: NodeRef<html::Textarea> = create_node_ref();
	let avatar_url_ref: NodeRef<html::Input> = create_node_ref();
	let banner_url_ref: NodeRef<html::Input> = create_node_ref();

	let myself = cache::OBJECTS.get(&auth.userid.get_untracked().unwrap_or_default());
	let curr_display_name = myself.as_ref().and_then(|x| Some(x.name().ok()?.to_string())).unwrap_or_default();
	let curr_summary = myself.as_ref().and_then(|x| Some(x.summary().ok()?.to_string())).unwrap_or_default();
	let curr_icon = myself.as_ref().and_then(|x| Some(x.icon().get()?.url().id().ok()?.to_string())).unwrap_or_default();
	let curr_banner = myself.as_ref().and_then(|x| Some(x.image().get()?.url().id().ok()?.to_string())).unwrap_or_default();

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
						set_color_rgb.set(parse_hex(&val));
						set_color.set(val.clone());
						mock.accent_color = val;
						setter.set(mock);
				} />
			</p>
			<hr />
			<p><code title="unchecked elements won't show in timelines">filters</code></p>
			<ul>
					<li><span title="replies to other posts"><input type="checkbox" prop:checked=get_cfg!(filter replies) on:input=set_cfg!(filter replies) />" replies"</span></li>
					<li><span title="like activities"><input type="checkbox" prop:checked=get_cfg!(filter likes) on:input=set_cfg!(filter likes) />" likes"</span></li>
					<li><span title="create activities with object"><input type="checkbox" prop:checked=get_cfg!(filter creates) on:input=set_cfg!(filter creates)/>" creates"</span></li>
					<li><span title="update activities, to objects or actors"><input type="checkbox" prop:checked=get_cfg!(filter updates) on:input=set_cfg!(filter updates)/>" updates"</span></li>
					<li><span title="delete activities"><input type="checkbox" prop:checked=get_cfg!(filter deletes) on:input=set_cfg!(filter deletes)/>" deletes"</span></li>
					<li><span title="announce activities with object"><input type="checkbox" prop:checked=get_cfg!(filter announces) on:input=set_cfg!(filter announces) />" announces"</span></li>
					<li><span title="follow, accept and reject activities"><input type="checkbox" prop:checked=get_cfg!(filter follows) on:input=set_cfg!(filter follows) />" follows"</span></li>
					<li><span title="objects without a related activity to display"><input type="checkbox" prop:checked=get_cfg!(filter orphans) on:input=set_cfg!(filter orphans) />" fetched"</span></li>
			</ul>
			<hr />
			<div class="border ma-2 pa-1">
				<code class="center cw color mb-1">update profile</code>
				<div class="col-side mb-0">display name</div>
				<div class="col-main">
					<input class="w-100" type="text" node_ref=display_name_ref placeholder="bmdieGo=" value={curr_display_name}/>
				</div>

				<div class="col-side mb-0">avatar url</div>
				<div class="col-main">
					<input class="w-100" type="text" node_ref=avatar_url_ref placeholder="https://cdn.alemi.dev/social/circle-square.png" value={curr_icon} />
				</div>

				<div class="col-side mb-0">banner url</div>
				<div class="col-main">
					<input class="w-100" type="text" node_ref=banner_url_ref placeholder="https://cdn.alemi.dev/social/gradient.png" value={curr_banner} />
				</div>

				<div class="col-side mb-0">summary</div>
				<div class="col-main">
					<textarea class="w-100" node_ref=summary_ref placeholder="when you lose control of yourself, who's controlling you?">{curr_summary}</textarea>
				</div>

				<input class="w-100" type="submit" value="submit"
					on:click=move|e| {
						e.prevent_default();
						let display_name = display_name_ref.get()
							.map(|x| x.value())
							.filter(|x| !x.is_empty());

						let summary = summary_ref.get()
							.map(|x| x.value())
							.filter(|x| !x.is_empty());

						let avatar = avatar_url_ref.get()
							.map(|x| x.value())
							.filter(|x| !x.is_empty())
							.map(|x|
								apb::new()
									.set_document_type(Some(apb::DocumentType::Image))
									.set_url(apb::Node::link(x))
							);

						let banner = banner_url_ref.get()
							.map(|x| x.value())
							.filter(|x| !x.is_empty())
							.map(|x|
								apb::new()
									.set_document_type(Some(apb::DocumentType::Image))
									.set_url(apb::Node::link(x))
							);
						
						let id = auth.userid.get_untracked().unwrap_or_default();
						let Some(me) = cache::OBJECTS.get(&id) else {
							tracing::error!("self user not in cache! can't update");
							return;
						};

						let payload = apb::new()
							.set_activity_type(Some(apb::ActivityType::Update))
							.set_to(apb::Node::links(vec![apb::target::PUBLIC.to_string(), format!("{id}/followers")]))
							.set_object(apb::Node::object(
								(*me).clone()
									.set_name(display_name.as_deref())
									.set_summary(summary.as_deref())
									.set_icon(apb::Node::maybe_object(avatar))
									.set_image(apb::Node::maybe_object(banner))
									.set_published(Some(chrono::Utc::now()))
							));

						spawn_local(async move {
							if let Err(e) = Http::post(&format!("{id}/outbox"), &payload, auth).await {
								tracing::error!("could not send update activity: {e}");
							}
						});
					}
				/>
			</div>
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
