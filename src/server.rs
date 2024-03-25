use std::sync::Arc;

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Context(Arc<ContextInner>);
struct ContextInner {
	db: DatabaseConnection,
	domain: String,
}

#[macro_export]
macro_rules! url {
	($ctx:expr, $($args: tt)*) => {
		format!("{}{}", $ctx.base(), format!($($args)*))
	};
}

impl Context {
	pub fn new(db: DatabaseConnection, mut domain: String) -> Self {
		if !domain.starts_with("http") {
			domain = format!("https://{domain}");
		}
		if domain.ends_with('/') {
			domain.replace_range(domain.len()-1.., "");
		}
		Context(Arc::new(ContextInner { db, domain }))
	}

	pub fn db(&self) -> &DatabaseConnection {
		&self.0.db
	}

	pub fn base(&self) -> &str {
		&self.0.domain
	}

	pub fn uri(&self, entity: &str, id: String) -> String {
		if id.starts_with("http") { id } else {
			format!("{}/{}/{}", self.0.domain, entity, id)
		}
	}

	/// get full user id uri
	pub fn uid(&self, id: String) -> String {
		self.uri("users", id)
	}

	/// get full object id uri
	pub fn oid(&self, id: String) -> String {
		self.uri("objects", id)
	}

	/// get full activity id uri
	pub fn aid(&self, id: String) -> String {
		self.uri("activities", id)
	}

	/// get bare id, usually an uuid but unspecified
	pub fn id(&self, id: String) -> String {
		if id.starts_with(&self.0.domain) {
			id.split('/').last().unwrap_or("").to_string()
		} else {
			id
		}
	}

	pub fn server(id: &str) -> String {
		id
			.replace("https://", "")
			.replace("http://", "")
			.split('/')
			.next()
			.unwrap_or("")
			.to_string()
	}
}
