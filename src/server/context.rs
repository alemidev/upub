use std::sync::Arc;

use apb::{BaseMut, CollectionMut, CollectionPageMut};
use openssl::rsa::Rsa;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, SelectColumns, Set};

use crate::{model, routes::activitypub::jsonld::LD};

use super::{dispatcher::Dispatcher, fetcher::Fetcher};


#[derive(Clone)]
pub struct Context(Arc<ContextInner>);
struct ContextInner {
	db: DatabaseConnection,
	domain: String,
	protocol: String,
	fetcher: Fetcher,
	dispatcher: Dispatcher,
	// TODO keep these pre-parsed
	app: model::application::Model,
}

#[macro_export]
macro_rules! url {
	($ctx:expr, $($args: tt)*) => {
		format!("{}{}{}", $ctx.protocol(), $ctx.base(), format!($($args)*))
	};
}

impl Context {

	// TODO slim constructor down, maybe make a builder?
	pub async fn new(db: DatabaseConnection, mut domain: String) -> crate::Result<Self> {
		let protocol = if domain.starts_with("http://")
		{ "http://" } else { "https://" }.to_string();
		if domain.ends_with('/') {
			domain.replace_range(domain.len()-1.., "");
		}
		if domain.starts_with("http") {
			domain = domain.replace("https://", "").replace("http://", "");
		}
		let dispatcher = Dispatcher::new();
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

		let fetcher = Fetcher::new(db.clone(), domain.clone(), app.private_key.clone());

		Ok(Context(Arc::new(ContextInner {
			db, domain, protocol, app, fetcher, dispatcher,
		})))
	}

	pub fn app(&self) -> &model::application::Model {
		&self.0.app
	}

	pub fn db(&self) -> &DatabaseConnection {
		&self.0.db
	}

	pub fn base(&self) -> &str {
		&self.0.domain
	}

	pub fn protocol(&self) -> &str {
		&self.0.protocol
	}

	pub fn uri(&self, entity: &str, id: String) -> String {
		if id.starts_with("http") { id } else {
			format!("{}{}/{}/{}", self.0.protocol, self.0.domain, entity, id)
		}
	}

	pub fn fetch(&self) -> &Fetcher {
		&self.0.fetcher
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

	pub fn is_local(&self, id: &str) -> bool {
		// TODO consider precalculating once this format!
		id.starts_with(&format!("{}{}", self.0.protocol, self.0.domain))
	}

	pub async fn expand_addressing(&self, targets: Vec<String>) -> crate::Result<Vec<String>> {
		let mut out = Vec::new();
		for target in targets {
			if target.ends_with("/followers") {
				let target_id = target.replace("/followers", "");
				model::relation::Entity::find()
					.filter(Condition::all().add(model::relation::Column::Following.eq(target_id)))
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

	pub async fn address_to(&self, aid: &str, oid: Option<&str>, targets: &[String]) -> crate::Result<()> {
		let local_activity = self.is_local(aid);
		let addressings : Vec<model::addressing::ActiveModel> = targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| !to.ends_with("/followers"))
			.filter(|to| local_activity || self.is_local(to))
			.map(|to| model::addressing::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				server: Set(Context::server(to)),
				actor: Set(to.to_string()),
				activity: Set(aid.to_string()),
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
		let deliveries : Vec<model::delivery::ActiveModel> = targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| Context::server(to) != self.base())
			.filter(|to| to != &apb::target::PUBLIC)
			.map(|to| model::delivery::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				actor: Set(from.to_string()),
				// TODO we should resolve each user by id and check its inbox because we can't assume
				// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
				target: Set(format!("{}/inbox", to)),
				activity: Set(aid.to_string()),
				created: Set(chrono::Utc::now()),
				not_before: Set(chrono::Utc::now()),
				attempt: Set(0),
			})
			.collect();

		if !deliveries.is_empty() {
			model::delivery::Entity::insert_many(deliveries)
				.exec(self.db())
				.await?;
		}

		self.0.dispatcher.wakeup();

		Ok(())
	}

	// TODO should probs not be here
	pub fn ap_collection(&self, id: &str, total_items: Option<u64>) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(id))
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(format!("{id}/page")))
			.set_total_items(total_items)
	}

	// TODO should probs not be here
	pub fn ap_collection_page(&self, id: &str, offset: u64, limit: u64, items: Vec<serde_json::Value>) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&format!("{id}?offset={offset}")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollectionPage))
			.set_part_of(apb::Node::link(id.replace("/page", "")))
			.set_next(apb::Node::link(format!("{id}?offset={}", offset+limit)))
			.set_ordered_items(apb::Node::Array(items))
	}

	pub async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()> {
		let addressed = self.expand_addressing(activity_targets).await?;
		self.address_to(aid, oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}
}