use std::{collections::BTreeSet, sync::Arc};

use openssl::rsa::Rsa;
use sea_orm::{ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, SelectColumns, Set};

use crate::{config::Config, errors::UpubError, model, server::fetcher::Fetcher};
use uriproxy::UriClass;

use super::dispatcher::Dispatcher;


#[derive(Clone)]
pub struct Context(Arc<ContextInner>);
struct ContextInner {
	db: DatabaseConnection,
	config: Config,
	domain: String,
	protocol: String,
	base_url: String,
	dispatcher: Dispatcher,
	// TODO keep these pre-parsed
	app: model::actor::Model,
	pkey: String,
	relay: Relays,
}

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
	pub async fn new(db: DatabaseConnection, mut domain: String, config: Config) -> crate::Result<Self> {
		let protocol = if domain.starts_with("http://")
		{ "http://" } else { "https://" }.to_string();
		if domain.ends_with('/') {
			domain.replace_range(domain.len()-1.., "");
		}
		if domain.starts_with("http") {
			domain = domain.replace("https://", "").replace("http://", "");
		}
		let dispatcher = Dispatcher::default();
		for _ in 0..1 { // TODO customize delivery workers amount
			dispatcher.spawn(db.clone(), domain.clone(), 30); // TODO ew don't do it this deep and secretly!!
		}
		let base_url = format!("{}{}", protocol, domain);
		let app = match model::actor::Entity::find_by_ap_id(&base_url).one(&db).await? {
			Some(model) => model,
			None => {
				tracing::info!("generating application keys");
				let rsa = Rsa::generate(2048)?;
				let privk = std::str::from_utf8(&rsa.private_key_to_pem()?)?.to_string();
				let pubk = std::str::from_utf8(&rsa.public_key_to_pem()?)?.to_string();
				let system = model::actor::ActiveModel {
					internal: NotSet,
					id: Set(base_url.clone()),
					domain: Set(domain.clone()),
					preferred_username: Set(domain.clone()),
					actor_type: Set(apb::ActorType::Application),
					private_key: Set(Some(privk)),
					public_key: Set(pubk),
					following: Set(None),
					following_count: Set(0),
					followers: Set(None),
					followers_count: Set(0),
					statuses_count: Set(0),
					summary: Set(Some("micro social network, federated".to_string())),
					name: Set(Some("μpub".to_string())),
					image: Set(None),
					icon: Set(Some("https://cdn.alemi.dev/social/circle-square.png".to_string())),
					inbox: Set(Some(format!("{base_url}/inbox"))),
					shared_inbox: Set(Some(format!("{base_url}/inbox"))),
					outbox: Set(Some(format!("{base_url}/outbox"))),
					published: Set(chrono::Utc::now()),
					updated: Set(chrono::Utc::now()),
				};
				model::actor::Entity::insert(system).exec(&db).await?;
				// sqlite doesn't resurn last inserted id so we're better off just querying again, it's just one time
				model::actor::Entity::find().one(&db).await?.expect("could not find app config just inserted")
			}
		};

		// TODO maybe we could provide a more descriptive error...
		let pkey = app.private_key.as_deref().ok_or_else(UpubError::internal_server_error)?.to_string();

		let relay_sinks = model::relation::Entity::followers(&app.id, &db).await?;
		let relay_sources = model::relation::Entity::following(&app.id, &db).await?;

		let relay = Relays {
			sources: BTreeSet::from_iter(relay_sources),
			sinks: BTreeSet::from_iter(relay_sinks),
		};

		Ok(Context(Arc::new(ContextInner {
			base_url, db, domain, protocol, app, dispatcher, config, pkey, relay,
		})))
	}

	pub fn app(&self) -> &model::actor::Model {
		&self.0.app
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

	/// get full user id uri
	pub fn uid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::User, id)
	}

	/// get full object id uri
	pub fn oid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::Object, id)
	}

	/// get full activity id uri
	pub fn aid(&self, id: &str) -> String {
		uriproxy::uri(self.base(), UriClass::Activity, id)
	}

	// TODO remove this!!
	//#[deprecated = "context is id of first post in thread"]
	pub fn context_id(&self, id: &str) -> String {
		if id.starts_with("tag:") {
			return id.to_string();
		}
		uriproxy::uri(self.base(), UriClass::Context, id)
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

	pub async fn is_local_internal_object(&self, internal: i64) -> crate::Result<bool> {
		Ok(
			model::object::Entity::find_by_id(internal)
				.select_only()
				.select_column(model::object::Column::Internal)
				.one(self.db())
				.await?
				.is_some()
		)
	}

	pub async fn is_local_internal_activity(&self, internal: i64) -> crate::Result<bool> {
		Ok(
			model::activity::Entity::find_by_id(internal)
				.select_only()
				.select_column(model::activity::Column::Internal)
				.one(self.db())
				.await?
				.is_some()
		)
	}

	#[allow(unused)]
	pub async fn is_local_internal_actor(&self, internal: i64) -> crate::Result<bool> {
		Ok(
			model::actor::Entity::find_by_id(internal)
				.select_only()
				.select_column(model::actor::Column::Internal)
				.one(self.db())
				.await?
				.is_some()
		)
	}

	pub async fn expand_addressing(&self, targets: Vec<String>) -> crate::Result<Vec<String>> {
		let mut out = Vec::new();
		for target in targets {
			if target.ends_with("/followers") {
				let target_id = target.replace("/followers", "");
				model::relation::Entity::find()
					.filter(model::relation::Column::Following.eq(target_id))
					.select_only()
					.select_column(model::relation::Column::Follower)
					.into_tuple::<String>()
					.all(self.db())
					.await?
					.into_iter()
					.for_each(|x| out.push(x));
			} else {
				out.push(target);
			}
		}
		Ok(out)
	}

	pub async fn address_to(&self, aid: Option<i64>, oid: Option<i64>, targets: &[String]) -> crate::Result<()> {
		// TODO address_to became kind of expensive, with these two selects right away and then another
		//      select for each target we're addressing to... can this be improved??
		let local_activity = if let Some(x) = aid { self.is_local_internal_activity(x).await? } else { false };
		let local_object = if let Some(x) = oid { self.is_local_internal_object(x).await? } else { false };
		let mut addressing = Vec::new();
		for target in targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| !to.ends_with("/followers"))
			.filter(|to| local_activity || local_object || to.as_str() == apb::target::PUBLIC || self.is_local(to))
		{
			let (server, actor) = if target == apb::target::PUBLIC { (None, None) } else {
				(
					Some(model::instance::Entity::domain_to_internal(&Context::server(target), self.db()).await?),
					Some(model::actor::Entity::ap_to_internal(target, self.db()).await?),
				)
			};
			addressing.push(
				model::addressing::ActiveModel {
					internal: sea_orm::ActiveValue::NotSet,
					instance: Set(server),
					actor: Set(actor),
					activity: Set(aid),
					object: Set(oid),
					published: Set(chrono::Utc::now()),
				}
			);
		}

		if !addressing.is_empty() {
			model::addressing::Entity::insert_many(addressing)
				.exec(self.db())
				.await?;
		}

		Ok(())
	}

	pub async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> crate::Result<()> {
		let mut deliveries = Vec::new();
		for target in targets.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| Context::server(to) != self.domain())
			.filter(|to| to != &apb::target::PUBLIC)
		{
			// TODO fetch concurrently
			match self.fetch_user(target).await {
				Ok(model::actor::Model { inbox: Some(inbox), .. }) => deliveries.push(
					model::delivery::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						actor: Set(from.to_string()),
						// TODO we should resolve each user by id and check its inbox because we can't assume
						// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
						target: Set(inbox),
						activity: Set(aid.to_string()),
						published: Set(chrono::Utc::now()),
						not_before: Set(chrono::Utc::now()),
						attempt: Set(0),
					}
				),
				Ok(_) => tracing::error!("resolved target but missing inbox: '{target}', skipping delivery"),
				Err(e) => tracing::error!("failed resolving target inbox: {e}, skipping delivery to '{target}'"),
			}
		}

		if !deliveries.is_empty() {
			model::delivery::Entity::insert_many(deliveries)
				.exec(self.db())
				.await?;
		}

		self.0.dispatcher.wakeup();

		Ok(())
	}

	//#[deprecated = "should probably directly invoke address_to() since we most likely have internal ids at this point"]
	pub async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()> {
		let addressed = self.expand_addressing(activity_targets).await?;
		let internal_aid = model::activity::Entity::ap_to_internal(aid, self.db()).await?;
		let internal_oid = if let Some(o) = oid { Some(model::object::Entity::ap_to_internal(o, self.db()).await?) } else { None };
		self.address_to(Some(internal_aid), internal_oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}

	pub fn is_relay(&self, id: &str) -> bool {
		self.0.relay.sources.contains(id) || self.0.relay.sinks.contains(id)
	}
}
