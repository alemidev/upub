use std::{collections::BTreeSet, sync::Arc};

use openssl::rsa::Rsa;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, SelectColumns, Set};

use crate::{config::Config, model, server::fetcher::Fetcher};
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
	app: model::application::Model,
	relays: BTreeSet<String>,
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
		let app = match model::application::Entity::find().one(&db).await? {
			Some(model) => model,
			None => {
				tracing::info!("generating application keys");
				let rsa = Rsa::generate(2048)?;
				let privk = std::str::from_utf8(&rsa.private_key_to_pem()?)?.to_string();
				let pubk = std::str::from_utf8(&rsa.public_key_to_pem()?)?.to_string();
				let system = model::application::ActiveModel {
					id: sea_orm::ActiveValue::NotSet,
					private_key: sea_orm::ActiveValue::Set(privk.clone()),
					public_key: sea_orm::ActiveValue::Set(pubk.clone()),
					created: sea_orm::ActiveValue::Set(chrono::Utc::now()),
				};
				model::application::Entity::insert(system).exec(&db).await?;
				// sqlite doesn't resurn last inserted id so we're better off just querying again, it's just one time
				model::application::Entity::find().one(&db).await?.expect("could not find app config just inserted")
			}
		};

		let relays = model::relay::Entity::find()
			.select_only()
			.select_column(model::relay::Column::Id)
			.filter(model::relay::Column::Accepted.eq(true))
			.into_tuple::<String>()
			.all(&db)
			.await?;

		Ok(Context(Arc::new(ContextInner {
			base_url: format!("{}{}", protocol, domain),
			db, domain, protocol, app, dispatcher, config,
			relays: BTreeSet::from_iter(relays.into_iter()),
		})))
	}

	pub fn app(&self) -> &model::application::Model {
		&self.0.app
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
	pub fn context_id(&self, id: &str) -> String {
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

	pub async fn address_to(&self, aid: Option<&str>, oid: Option<&str>, targets: &[String]) -> crate::Result<()> {
		let local_activity = aid.map(|x| self.is_local(x)).unwrap_or(false);
		let local_object = oid.map(|x| self.is_local(x)).unwrap_or(false);
		let addressings : Vec<model::addressing::ActiveModel> = targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| !to.ends_with("/followers"))
			.filter(|to| local_activity || local_object || to.as_str() == apb::target::PUBLIC || self.is_local(to))
			.map(|to| model::addressing::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				server: Set(Context::server(to)),
				actor: Set(to.to_string()),
				activity: Set(aid.map(|x| x.to_string())),
				object: Set(oid.map(|x| x.to_string())),
				published: Set(chrono::Utc::now()),
			})
			.collect();

		if !addressings.is_empty() {
			model::addressing::Entity::insert_many(addressings)
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
				Ok(model::user::Model { inbox: Some(inbox), .. }) => deliveries.push(
					model::delivery::ActiveModel {
						id: sea_orm::ActiveValue::NotSet,
						actor: Set(from.to_string()),
						// TODO we should resolve each user by id and check its inbox because we can't assume
						// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
						target: Set(inbox),
						activity: Set(aid.to_string()),
						created: Set(chrono::Utc::now()),
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

	pub async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()> {
		let addressed = self.expand_addressing(activity_targets).await?;
		self.address_to(Some(aid), oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}

	pub fn is_relay(&self, id: &str) -> bool {
		self.0.relays.contains(id)
	}
}
