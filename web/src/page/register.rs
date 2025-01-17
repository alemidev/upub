use leptos::prelude::*;
use reqwest::Method;
use crate::prelude::*;

macro_rules! get_ref {
	($r:ident) => {
		$r.get().map(|x| x.value()).filter(|x| !x.is_empty())
	}
}

macro_rules! reset_ref {
	($r:ident) => {
		$r.get().map(|x| x.set_value(""))
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
	let username_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let password_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let display_name_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let summary_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let avatar_url_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let banner_url_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let (error, set_error) = signal(None);
	view! {
		<div class="two-col">
			<div class="border ma-2 pa-1">
				<form on:submit=move|ev| {
					ev.prevent_default();
					set_error.set(None);
					let email = get_ref!(username_ref);
					let password = get_ref!(password_ref);
					let display_name = get_ref!(display_name_ref);
					let summary = get_ref!(summary_ref);
					let avatar_url = get_ref!(avatar_url_ref);
					let banner_url = get_ref!(banner_url_ref);

					if email.is_none() || password.is_none() {
						set_error.set(Some("no credentials provided".to_string()));
						return;
					}

					leptos::task::spawn_local(async move {
						let payload = RegisterForm {
							username: email.unwrap_or_default(),
							password: password.unwrap_or_default(),
							display_name, summary, avatar_url, banner_url
						};
						match Http::request(Method::PUT, &format!("{URL_BASE}/auth"), Some(&payload), auth).await {
							Err(e) => set_error.set(Some(e.to_string())),
							Ok(res) => match res.error_for_status() {
								Err(e) => set_error.set(Some(e.to_string())),
								Ok(_) => {
									reset_ref!(username_ref);
									reset_ref!(password_ref);
									reset_ref!(display_name_ref);
									reset_ref!(summary_ref);
									reset_ref!(avatar_url_ref);
									reset_ref!(banner_url_ref);
									set_error.set(Some("registration successful! your user may need to be approved by an administrator before you can login".to_string()));
								},
							},
						}
					});
				} >
					<div class="col-side mb-0">username</div>
					<div class="col-main">
						<input class="w-100" type="text" node_ref=username_ref placeholder="doll" />
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
			<p>{error.get().map(|msg| view! { <blockquote>{msg}</blockquote> })}</p>
		</div>
	}
}
