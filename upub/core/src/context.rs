use std::{collections::BTreeSet, sync::Arc};

use sea_orm::{DatabaseConnection, DbErr, QuerySelect, SelectColumns};

use crate::{config::Config, model};
use uriproxy::UriClass;

#[derive(Clone)]
pub struct Context(Arc<ContextInner>);
struct ContextInner {
	db: DatabaseConnection,
	config: Config,
	domain: String,
	protocol: String,
	base_url: String,
	// TODO keep these pre-parsed
	actor: model::actor::Model,
	instance: model::instance::Model,
	pkey: String,
	waker: Option<Box<dyn WakerToken>>,
	#[allow(unused)] relay: Relays,
}

#[allow(unused)]
pub struct Relays {
	sources: BTreeSet<String>,
	sinks: BTreeSet<String>,
}

#[macro_export]
macro_rules! url {
	($ctx:expr, $($args: tt)*) => {
		format!("{}{}{}", $ctx.protocol(), $ctx.domain(), format!($($args)*))
	};
}

pub trait WakerToken: Sync + Send {
	fn wake(&self);
}

impl Context {

	// TODO slim constructor down, maybe make a builder?
	pub async fn new(db: DatabaseConnection, mut domain: String, config: Config, waker: Option<Box<dyn WakerToken>>) -> Result<Self, crate::init::InitError> {
		let protocol = if domain.starts_with("http://")
		{ "http://" } else { "https://" }.to_string();
		if domain.ends_with('/') {
			domain.replace_range(domain.len()-1.., "");
		}
		if domain.starts_with("http") {
			domain = domain.replace("https://", "").replace("http://", "");
		}
		let base_url = format!("{}{}", protocol, domain);

		let (actor, instance) = super::init::application(domain.clone(), base_url.clone(), &db).await?;

		// TODO maybe we could provide a more descriptive error...
		let pkey = actor.private_key.as_deref().ok_or_else(|| DbErr::RecordNotFound("application private key".into()))?.to_string();

		let relay_sinks = crate::Query::related(None, Some(actor.internal), false)
			.select_only()
			.select_column(crate::model::actor::Column::Id)
			.into_tuple::<String>()
			.all(&db)
			.await?;

		let relay_sources = crate::Query::related(Some(actor.internal), None, false)
			.select_only()
			.select_column(crate::model::actor::Column::Id)
			.into_tuple::<String>()
			.all(&db)
			.await?;

		let relay = Relays {
			sources: BTreeSet::from_iter(relay_sources),
			sinks: BTreeSet::from_iter(relay_sinks),
		};

		Ok(Context(Arc::new(ContextInner {
			base_url, db, domain, protocol, actor, instance, config, pkey, relay, waker,
		})))
	}

	pub fn actor(&self) -> &model::actor::Model {
		&self.0.actor
	}

	#[allow(unused)]
	pub fn instance(&self) -> &model::instance::Model {
		&self.0.instance
	}

	pub fn pkey(&self) -> &str {
		&self.0.pkey
	}

	pub fn db(&self) -> &DatabaseConnection {
		&self.0.db
	}

	pub fn cfg(&self) -> &Config {
		&self.0.config
	}

	pub fn domain(&self) -> &str {
		&self.0.domain
	}

	pub fn protocol(&self) -> &str {
		&self.0.protocol
	}

	pub fn base(&self) -> &str {
		&self.0.base_url
	}

	pub fn ap<T: crate::ext::IntoActivityPub>(&self, x: T) -> serde_json::Value {
		x.into_activity_pub_json(self)
	}

	pub fn new_id() -> String {
		uuid::Uuid::new_v4().to_string()
	}

	/// get full user id uri
	pub fn uid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::Actor, id)
	}

	/// get full object id uri
	pub fn oid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::Object, id)
	}

	/// get full activity id uri
	pub fn aid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::Activity, id)
	}

	/// get bare id, which is uuid for local stuff and +{uri|base64} for remote stuff
	pub fn id(&self, full_id: &str) -> String {
		if self.is_local(full_id) {
			uriproxy::decompose(full_id)
		} else {
			uriproxy::compact(full_id)
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

	pub fn is_local(&self, id: &str) -> bool {
		id.starts_with(self.base())
	}

	pub async fn find_internal(&self, id: &str) -> Result<Option<Internal>, DbErr> {
		if let Some(internal) = model::object::Entity::ap_to_internal(id, self.db()).await? {
			return Ok(Some(Internal::Object(internal)));
		}

		if let Some(internal) = model::activity::Entity::ap_to_internal(id, self.db()).await? {
			return Ok(Some(Internal::Activity(internal)));
		}

		if let Some(internal) = model::actor::Entity::ap_to_internal(id, self.db()).await? {
			return Ok(Some(Internal::Actor(internal)));
		}

		Ok(None)
	}

	pub fn wake_workers(&self) {
		if let Some(ref waker) = self.0.waker {
			waker.wake();
		}
	}

	#[allow(unused)]
	pub fn is_relay(&self, id: &str) -> bool {
		self.0.relay.sources.contains(id) || self.0.relay.sinks.contains(id)
	}
}

pub enum Internal {
	Object(i64),
	Activity(i64),
	Actor(i64),
}
