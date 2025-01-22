use base64::{Engine, prelude::BASE64_URL_SAFE};
use hmac::Mac;


pub type Signature = hmac::Hmac<sha2::Sha256>;

pub trait Cloaker {
	fn secret(&self) -> &str;

	fn cloak(&self, url: &str) -> (String, String) {
		let mut hmac = Signature::new_from_slice(self.secret().as_bytes())
			.expect("invalid length for hmac key, cannot cloak");
		hmac.update(url.as_bytes());
		let sig = BASE64_URL_SAFE.encode(hmac.finalize().into_bytes());
		let url = BASE64_URL_SAFE.encode(url);
		(sig, url)
	}

	fn uncloak(&self, signature: &str, url: &str) -> Option<String> {
		let mut hmac = Signature::new_from_slice(self.secret().as_bytes())
			.expect("invalid length for hmac key, cannot cloak");

		let sig = BASE64_URL_SAFE.decode(signature).ok()?;
		let url = std::str::from_utf8(&BASE64_URL_SAFE.decode(url).ok()?).ok()?.to_string();

		hmac.update(url.as_bytes());
		hmac.verify_slice(&sig).ok()?;

		Some(url)
	}

	fn cloaked(&self, url: &str) -> String;
}

impl Cloaker for crate::Context {
	fn secret(&self) -> &str {
		&self.cfg().security.proxy_secret
	}

	fn cloaked(&self, url: &str) -> String {
		// pre-cloaked: uncloak without validating and re-cloak
		let cloak_base = crate::url!(self, "/proxy/");
		if url.starts_with(&cloak_base) {
			if let Some((_sig, url_b64)) = url.replace(&cloak_base, "").split_once("/") {
				if let Some(actual_url) = BASE64_URL_SAFE.decode(url_b64).ok()
					.and_then(|x| std::str::from_utf8(&x).ok().map(|x| x.to_string()))
				{
					let (sig, url) = self.cloak(&actual_url);
					return crate::url!(self, "/proxy/{sig}/{url}");
				}
			}
		}

		let (sig, url) = self.cloak(url);
		crate::url!(self, "/proxy/{sig}/{url}")
	}
}

// TODO this shouldnt sit in bare context.rs but also having it here is weird!!
impl crate::Context {
	pub fn sanitize(&self, text: &str) -> String {
		let _ctx = self.clone();
		mdhtml::Sanitizer::new(
			Box::new(move |txt| {
				if _ctx.is_local(txt) {
					txt.to_string()
				} else {
					_ctx.cloaked(txt)
				}
			})
		)
			.html(text)
	}
}
