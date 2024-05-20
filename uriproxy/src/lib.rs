use base64::Engine;

#[derive(Clone, Copy)]
pub enum UriClass {
	User,
	Object,
	Activity,
	Context,
}

impl AsRef<str> for UriClass {
	fn as_ref(&self) -> &str {
		match self {
			Self::User => "users",
			Self::Object => "objects",
			Self::Activity => "activities",
			Self::Context => "context",
		}
	}
}

/// unpack uri in id if valid, otherwise compose full uri with "{base}/{entity}/{id}"
pub fn uri(base: &str, entity: UriClass, id: &str) -> String {
	if let Some(bare_id) = get_nth_uri_element(id) {
		if bare_id.starts_with('~') {
			if let Ok(bytes) = base64::prelude::BASE64_STANDARD.decode(bare_id.replacen('~', "", 1)) {
				if let Ok(uri) = std::str::from_utf8(&bytes) {
					return uri.to_string();
				}
			}
		}
	}
	format!("{}/{}/{}", base, entity.as_ref(), id)
}

fn get_nth_uri_element(uri: &str) -> Option<String> {
	uri       //  https://example.org/users/test/followers/page?offset=42
		.split('/') //  ['https:', '', 'example.org', 'users', 'test', 'followers', 'page?offset=42' ]
		.nth(4)     //  'test'
		.map(|x| x.to_string())
}

/// decompose local id constructed by uri() fn
pub fn decompose_id(full_id: &str) -> String {
	get_nth_uri_element(full_id).unwrap_or_default()
}

/// encode with base64 remote url and prefix it with ~
pub fn compact_id(uri: &str) -> String {
	let encoded = base64::prelude::BASE64_STANDARD.encode(uri.as_bytes());
	format!("~{encoded}")
}
