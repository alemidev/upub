use std::{collections::BTreeSet, sync::Arc};

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{config::Config, ext::AnyQuery, model};
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

impl Context {

	// TODO slim constructor down, maybe make a builder?
	pub async fn new(db: DatabaseConnection, mut domain: String, config: Config) -> Result<Self, crate::init::InitError> {
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

		let relay_sinks = model::relation::Entity::followers(&actor.id, &db).await?.ok_or_else(|| DbErr::RecordNotFound(actor.id.clone()))?;
		let relay_sources = model::relation::Entity::following(&actor.id, &db).await?.ok_or_else(|| DbErr::RecordNotFound(actor.id.clone()))?;

		let relay = Relays {
			sources: BTreeSet::from_iter(relay_sources),
			sinks: BTreeSet::from_iter(relay_sinks),
		};

		Ok(Context(Arc::new(ContextInner {
			base_url, db, domain, protocol, actor, instance, config, pkey, relay,
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
			uriproxy::decompose_id(full_id)
		} else {
			uriproxy::compact_id(full_id)
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

	pub async fn is_local_internal_object(&self, internal: i64) -> Result<bool, DbErr> {
		model::object::Entity::find()
			.filter(model::object::Column::Internal.eq(internal))
			.select_only()
			.select_column(model::object::Column::Internal)
			.into_tuple::<i64>()
			.any(self.db())
			.await
	}

	pub async fn is_local_internal_activity(&self, internal: i64) -> Result<bool, DbErr> {
		model::activity::Entity::find()
			.filter(model::activity::Column::Internal.eq(internal))
			.select_only()
			.select_column(model::activity::Column::Internal)
			.into_tuple::<i64>()
			.any(self.db())
			.await
	}

	#[allow(unused)]
	pub async fn is_local_internal_actor(&self, internal: i64) -> Result<bool, DbErr> {
		model::actor::Entity::find()
			.filter(model::actor::Column::Internal.eq(internal))
			.select_only()
			.select_column(model::actor::Column::Internal)
			.into_tuple::<i64>()
			.any(self.db())
			.await
	}

	#[allow(unused)]
	pub fn is_relay(&self, id: &str) -> bool {
		self.0.relay.sources.contains(id) || self.0.relay.sinks.contains(id)
	}
}
