use base64::Engine;

#[derive(Clone, Copy)]
pub enum UriClass {
	Actor,
	Object,
	Activity,
	Hashtag,
}

impl AsRef<str> for UriClass {
	fn as_ref(&self) -> &str {
		match self {
			Self::Actor => "actors",
			Self::Object => "objects",
			Self::Activity => "activities",
			Self::Hashtag => "tags",
		}
	}
}

/// unpack uri in id if valid, otherwise compose full uri with "{base}/{entity}/{id}"
pub fn uri(base: &str, entity: UriClass, id: &str) -> String {
	if id.starts_with("https://") || id.starts_with("http://") {
		return id.to_string();
	}

	if id.starts_with('+') { // ready-to-use base64-encoded id
		if let Some(expanded) = expand(id) {
			return expanded;
		}
	}

	format!("{}/{}/{}", base, entity.as_ref(), id)
}

/// decompose local id constructed by uri() fn
pub fn decompose(full_id: &str) -> String {
		full_id       //  https://example.org/actors/test/followers/page?offset=42
			.replace("https://", "")
			.replace("http://", "")
			.split('/') //  ['example.org', 'actors', 'test', 'followers', 'page?offset=42' ]
			.nth(2)     //  'test'
			.unwrap_or("")
			.to_string()
}

pub fn expand(uri: &str) -> Option<String> {
	if let Ok(bytes) = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(uri.replacen('+', "", 1)) {
		if let Ok(uri) = std::str::from_utf8(&bytes) {
			return Some(uri.to_string());
		}
	}
	None
}

/// encode with base64 remote url and prefix it with +
pub fn compact(uri: &str) -> String {
	let encoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(uri.as_bytes());
	format!("+{encoded}")
}
