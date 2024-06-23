use leptos::*;
use reqwest::Method;
use crate::prelude::*;

macro_rules! get_ref {
	($r:ident) => {
		$r.get().map(|x| x.value()).filter(|x| !x.is_empty())
	}
}

// TODO this should get moved in a common crate so its not duplicated across FE/BE
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisterForm {
	username: String,
	password: String,
	display_name: Option<String>,
	summary: Option<String>,
	avatar_url: Option<String>,
	banner_url: Option<String>,
}

#[component]
pub fn RegisterPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	let display_name_ref: NodeRef<html::Input> = create_node_ref();
	let summary_ref: NodeRef<html::Input> = create_node_ref();
	let avatar_url_ref: NodeRef<html::Input> = create_node_ref();
	let banner_url_ref: NodeRef<html::Input> = create_node_ref();
	let (error, set_error) = create_signal(None);
	view! {
		<div class="two-col">
			<div class="border ma-2 pa-1">
				<form on:submit=move|ev| {
					ev.prevent_default();
					let email = get_ref!(username_ref);
					let password = get_ref!(password_ref);
					let display_name = get_ref!(display_name_ref);
					let summary = get_ref!(summary_ref);
					let avatar_url = get_ref!(avatar_url_ref);
					let banner_url = get_ref!(banner_url_ref);

					if email.is_none() || password.is_none() {
						set_error.set(Some(
							view! { <blockquote>no credentials provided</blockquote> }
						));
					}

					spawn_local(async move {
						let payload = RegisterForm {
							username: email.unwrap_or_default(),
							password: password.unwrap_or_default(),
							display_name, summary, avatar_url, banner_url
						};
						if let Err(e) = Http::request(Method::PUT, &format!("{URL_BASE}/auth"), Some(&payload), auth).await {
							set_error.set(Some(
								view! { <blockquote>{e.to_string()}</blockquote> }
							));
						}
					});
				} >
					<div class="col-side mb-0">username</div>
					<div class="col-main">
						<input class="w-100" type="email" node_ref=username_ref placeholder="doll" />
					</div>

					<div class="col-side mb-0">password</div>
					<div class="col-main">
						<input class="w-100" type="password" node_ref=password_ref placeholder="±·ì¥ì¤uª]*«P³.ÐvkÏÚ;åÍì§ÕºöAQ¿SnÔý" />
					</div>

					<div class="col-side mb-0"><hr /></div>
					<div class="col-main"><hr class="hidden-on-mobile" /></div>

					<div class="col-side mb-0">display name</div>
					<div class="col-main">
						<input class="w-100" type="text" node_ref=display_name_ref placeholder="bmdieGo="/>
					</div>

					<div class="col-side mb-0">summary</div>
					<div class="col-main">
						<input class="w-100" type="text" node_ref=summary_ref placeholder="when you lose control of yourself, who's controlling you?" />
					</div>

					<div class="col-side mb-0">avatar url</div>
					<div class="col-main">
						<input class="w-100" type="text" node_ref=avatar_url_ref placeholder="https://cdn.alemi.dev/social/circle-square.png" />
					</div>

					<div class="col-side mb-0">banner url</div>
					<div class="col-main">
						<input class="w-100" type="text" node_ref=banner_url_ref placeholder="https://cdn.alemi.dev/social/gradient.png" />
					</div>

					<div class="col-side mb-0"><hr /></div>
					<div class="col-main"><hr class="hidden-on-mobile" /></div>

					<input class="w-100" type="submit" value="register" />
				</form>
			</div>
			<p>{error}</p>
		</div>
	}
}
