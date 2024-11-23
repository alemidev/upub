use apb::{ActivityMut, Base, BaseMut, Object, ObjectMut};

use leptos::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct ReplyControls {
	pub context: RwSignal<Option<String>>,
	pub reply_to: RwSignal<Option<String>>,
}

impl ReplyControls {
	pub fn reply(&self, oid: &str) {
		if let Some(obj) = cache::OBJECTS.get(oid) {
			self.context.set(obj.context().id().ok());
			self.reply_to.set(obj.id().ok().map(|x| x.to_string()));
		}
	}

	pub fn clear(&self) {
		self.context.set(None);
		self.reply_to.set(None);
	}
}

fn post_author(post_id: &str) -> Option<crate::Object> {
	let usr = cache::OBJECTS.get(post_id)?.attributed_to().id().ok()?;
	cache::OBJECTS.get(&usr)
}

#[derive(Clone)]
struct MentionMatch {
	href: String,
	name: String,
	domain: String,
}

pub type PrivacyControl = ReadSignal<Privacy>;

#[derive(Debug, Clone, Copy)]
pub enum Privacy {
	Broadcast = 4,
	Public = 3,
	Private = 2,
	Direct = 1,
}

impl Privacy {
	pub fn is_public(&self) -> bool {
		match self {
			Self::Broadcast | Self::Public => true,
			_ => false,
		}
	}

	pub fn from_value(v: &str) -> Self {
		match v {
			"1" => Self::Direct,
			"2" => Self::Private,
			"3" => Self::Public,
			"4" => Self::Broadcast,
			_ => panic!("invalid value for privacy"),
		}
	}

	pub fn from_addressed(to: &[String], cc: &[String]) -> Self {
		if to.iter().any(|x| x == apb::target::PUBLIC) {
			return Self::Broadcast;
		}
		if cc.iter().any(|x| x == apb::target::PUBLIC) {
			return Self::Public;
		}
		if to.iter().any(|x| x.ends_with("/followers"))
		|| cc.iter().any(|x| x.ends_with("/followers")) {
			return Self::Private;
		}

		Self::Direct
	}

	pub fn icon(&self) -> &'static str {
		match self {
			Self::Broadcast => "📢",
			Self::Public => "🪩",
			Self::Private => "🔒",
			Self::Direct => "📨",
		}
	}

	pub fn address(&self, user: &str) -> (Vec<String>, Vec<String>) {
		match self {
			Self::Broadcast => (
				vec![apb::target::PUBLIC.to_string()],
				vec![format!("{URL_BASE}/actors/{user}/followers")],
			),
			Self::Public => (
				vec![],
				vec![apb::target::PUBLIC.to_string(), format!("{URL_BASE}/actors/{user}/followers")],
			),
			Self::Private => (
				vec![],
				vec![format!("{URL_BASE}/actors/{user}/followers")],
			),
			Self::Direct => (
				vec![],
				vec![],
			),
		}
	}
}

#[component]
pub fn PrivacySelector(setter: WriteSignal<Privacy>) -> impl IntoView {
	let privacy = use_context::<PrivacyControl>().expect("missing privacy context");
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<table class="align w-100">
			<tr>
				<td class="w-100">
					<input
						type="range"
						min="1"
						max="4"
						class="w-100"
						prop:value=move || privacy.get() as u8
						on:input=move |ev| {
							ev.prevent_default();
							setter.set(Privacy::from_value(&event_target_value(&ev)));
					} />
				</td>
				<td>
					{move || {
						let p = privacy.get();
						let (to, cc) = p.address(&auth.username());
						view! {
							<PrivacyMarker privacy=p to=&to cc=&cc big=true />
						}
					}}
				</td>
			</tr>
		</table>
	}
}

#[component]
pub fn PostBox(advanced: WriteSignal<bool>) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let privacy = use_context::<PrivacyControl>().expect("missing privacy context");
	let reply = use_context::<ReplyControls>().expect("missing reply controls");
	let (posting, set_posting) = create_signal(false);
	let (error, set_error) = create_signal(None);
	let (content, set_content) = create_signal("".to_string());
	let summary_ref: NodeRef<html::Input> = create_node_ref();

	// TODO is this too abusive with resources? im even checking if TLD exists...
	let mentions = create_local_resource(
		move || content.get(),
		move |c| async move {
			let mut out = Vec::new();
			for word in c.split(' ') {
				if !word.starts_with('@') { break };
				let stripped = word.replacen('@', "", 1);
				if let Some((name, domain)) = stripped.split_once('@') {
					if let Some(tld) = domain.split('.').last() {
						if tld::exist(tld) {
							if let Some(uid) = cache::WEBFINGER.blocking_resolve(name, domain).await {
								out.push(MentionMatch { name: name.to_string(), domain: domain.to_string(), href: uid });
							}
						}
					}
				}
			}
			out
		},
	);

	view! {
		<div>
			{move ||
				reply.reply_to.get().map(|r| {
					let actor_strip = post_author(&r).map(|x| view! { <ActorStrip object=x /> });
					view! {
						<span class="nowrap">
							<span 
								class="cursor emoji emoji-btn mr-s ml-s"
								on:click=move|_| reply.clear()
								title={format!("> {r} | ctx: {}", reply.context.get().unwrap_or_default())}
							>
								"✒️"
							</span>
							{actor_strip}
							<small class="tiny ml-1">"["<a class="clean" title="remove reply" href="#" on:click=move |_| reply.clear() >reply</a>"]"</small>
						</span>
					}
				})
			}
			{move ||
				mentions.get()
					.map(|x| x.into_iter().map(|u| match cache::OBJECTS.get(&u.href) {
						Some(u) => view! { <span class="nowrap"><span class="emoji mr-s ml-s">"📨"</span><ActorStrip object=u /></span> }.into_view(),
						None => view! { <span class="nowrap"><span class="emoji mr-s ml-s">"📨"</span><a href={Uri::web(U::Actor, &u.href)}>{u.href}</a></span> }.into_view(),
					})
					.collect_view())
			}
			<table class="align w-100">
				<tr>
					<td><input type="checkbox" on:input=move |ev| advanced.set(event_target_checked(&ev)) title="toggle advanced controls" /></td>
					<td class="w-100"><input class="w-100" type="text" node_ref=summary_ref title="summary" /></td>
				</tr>
			</table>

			<textarea rows="6" class="w-100" title="content" placeholder="\n look at nothing\n  what do you see?"
				prop:value=content
				on:input=move |ev| set_content.set(event_target_value(&ev))
			></textarea>

			<button class="w-100" prop:disabled=posting type="button" style="height: 3em" on:click=move |_| {
				set_posting.set(true);
				spawn_local(async move {
					let summary = get_if_some(summary_ref);
					let content = content.get();
					let (mut to_vec, cc_vec) = privacy.get().address(&auth.username());
					let mut mention_tags : Vec<serde_json::Value> = mentions.get()
						.unwrap_or_default()
						.into_iter()
						.map(|x| {
							use apb::LinkMut;
							LinkMut::set_name(apb::new(), Some(format!("@{}@{}", x.name, x.domain))) // TODO ewww but name clashes
								.set_link_type(Some(apb::LinkType::Mention))
								.set_href(Some(x.href))
						})
						.collect();

					if let Some(r) = reply.reply_to.get() {
						if let Some(au) = post_author(&r) {
							if let Ok(uid) = au.id() {
								to_vec.push(uid.to_string());
								if let Ok(name) = au.name() {
									let domain = Uri::domain(&uid);
									mention_tags.push({
										use apb::LinkMut;
										LinkMut::set_name(apb::new(), Some(format!("@{}@{}", name, domain))) // TODO ewww but name clashes
											.set_link_type(Some(apb::LinkType::Mention))
											.set_href(Some(uid))
									});
								}
							}
						}
					}
					for mention in mentions.get().as_deref().unwrap_or(&[]) {
						to_vec.push(mention.href.clone());
					}
					let payload = apb::new()
						.set_object_type(Some(apb::ObjectType::Note))
						.set_summary(summary)
						.set_content(Some(content))
						.set_context(apb::Node::maybe_link(reply.context.get()))
						.set_in_reply_to(apb::Node::maybe_link(reply.reply_to.get()))
						.set_to(apb::Node::links(to_vec))
						.set_cc(apb::Node::links(cc_vec))
						.set_tag(apb::Node::array(mention_tags));
					match Http::post(&auth.outbox(), &payload, auth).await {
						Err(e) => set_error.set(Some(e.to_string())),
						Ok(()) => {
							set_error.set(None);
							if let Some(x) = summary_ref.get() { x.set_value("") }
							set_content.set("".to_string());
						},
					}
					set_posting.set(false);
				})
			} >post</button>

			{move|| error.get().map(|x| view! { <blockquote class="mt-s">{x}</blockquote> })}
		</div>
	}
}

#[component]
pub fn AdvancedPostBox(advanced: WriteSignal<bool>) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let (posting, set_posting) = create_signal(false);
	let (error, set_error) = create_signal(None);
	let (value, set_value) = create_signal("Like".to_string());
	let (embedded, set_embedded) = create_signal(false);
	let sensitive_ref: NodeRef<html::Input> = create_node_ref();
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let content_ref: NodeRef<html::Textarea> = create_node_ref();
	let context_ref: NodeRef<html::Input> = create_node_ref();
	let name_ref: NodeRef<html::Input> = create_node_ref();
	let reply_ref: NodeRef<html::Input> = create_node_ref();
	let to_ref: NodeRef<html::Input> = create_node_ref();
	let object_id_ref: NodeRef<html::Input> = create_node_ref();
	let bto_ref: NodeRef<html::Input> = create_node_ref();
	let cc_ref: NodeRef<html::Input> = create_node_ref();
	let bcc_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
							 
				<table class="align w-100">
					<tr>
						<td>
							<input type="checkbox" title="advanced" checked on:input=move |ev| {
								advanced.set(event_target_checked(&ev)) 
							}/>
						</td>
						<td class="w-100">
							<select class="w-100" on:change=move |ev| set_value.set(event_target_value(&ev))>
								<SelectOption value is="Create" />
								<SelectOption value is="Like" />
								<SelectOption value is="Follow" />
								<SelectOption value is="Announce" />
								<SelectOption value is="Accept" />
								<SelectOption value is="Reject" />
								<SelectOption value is="Undo" />
								<SelectOption value is="Delete" />
								<SelectOption value is="Update" />
							</select>
						</td>
						<td>
							<input type="checkbox" title="embedded object" on:input=move |ev| {
								set_embedded.set(event_target_checked(&ev)) 
							}/>
						</td>
					</tr>
				</table>

				<input class="w-100" type="text" node_ref=object_id_ref title="objectId" placeholder="objectId" />

				<div class:hidden=move|| !embedded.get()>
					<input class="w-100" type="text" node_ref=name_ref title="name" placeholder="name" />
					<input class="w-100" type="text" node_ref=context_ref title="context" placeholder="context" />
					<input class="w-100" type="text" node_ref=reply_ref title="inReplyTo" placeholder="inReplyTo" />

					<table class="align w-100">
						<tr>
							<td><input type="checkbox" title="sensitive" checked node_ref=sensitive_ref/>
									</td>
							<td class="w-100">
								<input class="w-100" type="text" node_ref=summary_ref title="summary" placeholder="summary" />
							</td>
						</tr>
					</table>

					<textarea rows="5" class="w-100" node_ref=content_ref title="content" placeholder="content" ></textarea>
				</div>

				<table class="w-100 align">
					<tr>
						<td class="w-66"><input class="w-100" type="text" node_ref=to_ref title="to" placeholder="to" value=apb::target::PUBLIC /></td>
						<td class="w-66"><input class="w-100" type="text" node_ref=bto_ref title="bto" placeholder="bto" /></td>
					</tr>
					<tr>
						<td class="w-33"><input class="w-100" type="text" node_ref=cc_ref title="cc" placeholder="cc" value=format!("{URL_BASE}/actors/{}/followers", auth.username()) /></td>
						<td class="w-33"><input class="w-100" type="text" node_ref=bcc_ref title="bcc" placeholder="bcc" /></td>
					</tr>
				</table>

				<button class="w-100" type="button" prop:disabled=posting on:click=move |_| {
					set_posting.set(true);
					spawn_local(async move {
						let content = content_ref.get().filter(|x| !x.value().is_empty()).map(|x| x.value());
						let summary = get_if_some(summary_ref);
						let name = get_if_some(name_ref);
						let context = get_if_some(context_ref);
						let reply = get_if_some(reply_ref);
						let object_id = get_if_some(object_id_ref);
						let to = get_vec_if_some(to_ref);
						let bto = get_vec_if_some(bto_ref);
						let cc = get_vec_if_some(cc_ref);
						let bcc = get_vec_if_some(bcc_ref);
						let payload = serde_json::Value::Object(serde_json::Map::default())
							.set_activity_type(Some(value.get().as_str().try_into().unwrap_or(apb::ActivityType::Create)))
							.set_to(apb::Node::links(to.clone()))
							.set_bto(apb::Node::links(bto.clone()))
							.set_cc(apb::Node::links(cc.clone()))
							.set_bcc(apb::Node::links(bcc.clone()))
							.set_object(
								if embedded.get() {
									apb::Node::object(
										serde_json::Value::Object(serde_json::Map::default())
											.set_id(object_id)
											.set_object_type(Some(apb::ObjectType::Note))
											.set_name(name)
											.set_summary(summary)
											.set_content(content)
											.set_in_reply_to(apb::Node::maybe_link(reply))
											.set_context(apb::Node::maybe_link(context))
											.set_to(apb::Node::links(to))
											.set_bto(apb::Node::links(bto))
											.set_cc(apb::Node::links(cc))
											.set_bcc(apb::Node::links(bcc))
									)
								} else {
									apb::Node::maybe_link(object_id)
								}
							);
						let target_url = format!("{URL_BASE}/actors/{}/outbox", auth.username());
						match Http::post(&target_url, &payload, auth).await {
							Err(e) => set_error.set(Some(e.to_string())),
							Ok(()) => set_error.set(None),
						}
						set_posting.set(false);
					})
				} >post</button>
			{move|| error.get().map(|x| view! { <blockquote class="mt-s">{x}</blockquote> })}
		</div>
	}
}

fn get_if_some(node: NodeRef<html::Input>) -> Option<String> {
	node.get()
		.map(|x| x.value())
		.filter(|x| !x.is_empty())
}

fn get_vec_if_some(node: NodeRef<html::Input>) -> Vec<String> {
	node.get()
		.map(|x| x.value())
		.filter(|x| !x.is_empty())
		.map(|x|
			x.split(',')
				.map(|x| x.to_string())
				.collect()
		).unwrap_or_default()
}

fn get_checked(node: NodeRef<html::Input>) -> bool {
	node.get()
		.map(|x| x.checked())
		.unwrap_or_default()
}

#[component]
fn SelectOption(is: &'static str, value: ReadSignal<String>) -> impl IntoView {
	view! {
		<option value=is selected=move || value.get() == is >
			{is}
		</option>
	}
}
