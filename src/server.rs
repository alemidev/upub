use std::{str::Utf8Error, sync::Arc};

use openssl::rsa::Rsa;
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns, Set};

use crate::{activitypub::{jsonld::LD, APOutbox, Addressed, CreationResult, PUBLIC_TARGET}, dispatcher::Dispatcher, errors::UpubError, fetcher::Fetcher, model};
use apb::{Activity, ActivityMut, BaseMut, CollectionMut, CollectionPageMut, CollectionType, Node, ObjectMut};

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

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
	#[error("database error: {0}")]
	Db(#[from] DbErr),

	#[error("openssl error: {0}")]
	OpenSSL(#[from] openssl::error::ErrorStack),

	#[error("invalid UTF8 PEM key: {0}")]
	UTF8Error(#[from] Utf8Error)
}

impl Context {

	// TODO slim constructor down, maybe make a builder?
	pub async fn new(db: DatabaseConnection, mut domain: String) -> Result<Self, ContextError> {
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

	pub async fn expand_addressing(&self, uid: &str, mut targets: Vec<String>) -> Result<Vec<String>, DbErr> {
		let following_addr = format!("{uid}/followers");
		if let Some(i) = targets.iter().position(|x| x == &following_addr) {
			targets.remove(i);
			model::relation::Entity::find()
				.filter(Condition::all().add(model::relation::Column::Following.eq(uid.to_string())))
				.select_only()
				.select_column(model::relation::Column::Follower)
				.into_tuple::<String>()
				.all(self.db())
				.await?
				.into_iter()
				.for_each(|x| targets.push(x));
		}
		Ok(targets)
	}

	pub async fn address_to(&self, aid: &str, oid: Option<&str>, targets: &[String]) -> Result<(), DbErr> {
		let addressings : Vec<model::addressing::ActiveModel> = targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| !to.ends_with("/followers"))
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

	pub async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> Result<(), DbErr> {
		let deliveries : Vec<model::delivery::ActiveModel> = targets
			.iter()
			.filter(|to| !to.is_empty())
			.filter(|to| Context::server(to) != self.base())
			.filter(|to| to != &PUBLIC_TARGET)
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
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_first(Node::link(format!("{id}/page")))
			.set_total_items(total_items)
	}

	// TODO should probs not be here
	pub fn ap_collection_page(&self, id: &str, offset: u64, limit: u64, items: Vec<serde_json::Value>) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&format!("{id}?offset={offset}")))
			.set_collection_type(Some(CollectionType::OrderedCollectionPage))
			.set_part_of(Node::link(id.replace("/page", "")))
			.set_next(Node::link(format!("{id}?offset={}", offset+limit)))
			.set_ordered_items(Node::Array(items))
	}

	pub async fn dispatch(&self, uid: &str, activity_targets: Vec<String>, aid: &str, oid: Option<&str>) -> crate::Result<()> {
		let addressed = self.expand_addressing(uid, activity_targets).await?;
		self.address_to(aid, oid, &addressed).await?;
		self.deliver_to(aid, uid, &addressed).await?;
		Ok(())
	}
}

#[axum::async_trait]
impl APOutbox for Context {
	async fn post_note(&self, uid: String, object: serde_json::Value) -> crate::Result<String> {
		let oid = self.oid(uuid::Uuid::new_v4().to_string());
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = object.addressed();
		let object_model = model::object::Model::new(
			&object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		let activity_model = model::activity::Model {
			id: aid.clone(),
			activity_type: apb::ActivityType::Create,
			actor: uid.clone(),
			object: Some(oid.clone()),
			target: None,
			cc: object_model.cc.clone(),
			bcc: object_model.bcc.clone(),
			to: object_model.to.clone(),
			bto: object_model.bto.clone(),
			published: object_model.published,
		};

		model::object::Entity::insert(object_model.into_active_model())
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}

	async fn post_activity(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let Some(object) = activity.object().extract() else {
			return Err(UpubError::bad_request());
		};

		let oid = self.oid(uuid::Uuid::new_v4().to_string());
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let mut object_model = model::object::Model::new(
			&object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		let mut activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		object_model.to = activity_model.to.clone();
		object_model.bto = activity_model.bto.clone();
		object_model.cc = activity_model.cc.clone();
		object_model.bcc = activity_model.bcc.clone();
		activity_model.object = Some(oid.clone());

		model::object::Entity::insert(object_model.into_active_model())
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}
		

	async fn like(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let Some(oid) = activity.object().id() else {
			return Err(StatusCode::BAD_REQUEST.into());
		};
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_published(Some(chrono::Utc::now()))
				.set_actor(Node::link(uid.clone()))
		)?;

		let like_model = model::like::ActiveModel {
			actor: Set(uid.clone()),
			likes: Set(oid),
			date: Set(chrono::Utc::now()),
			..Default::default()
		};
		model::like::Entity::insert(like_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn follow(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		if activity.object().id().is_none() {
			return Err(StatusCode::BAD_REQUEST.into());
		}

		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn accept(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		if activity.object().id().is_none() {
			return Err(StatusCode::BAD_REQUEST.into());
		}
		let Some(accepted_id) = activity.object().id() else {
			return Err(StatusCode::BAD_REQUEST.into());
		};
		let Some(accepted_activity) = model::activity::Entity::find_by_id(accepted_id)
			.one(self.db()).await?
		else {
			return Err(StatusCode::NOT_FOUND.into());
		};

		match accepted_activity.activity_type {
			apb::ActivityType::Follow => {
				model::relation::Entity::insert(
					model::relation::ActiveModel {
						follower: Set(accepted_activity.actor), following: Set(uid.clone()),
						..Default::default()
					}
				).exec(self.db()).await?;
			},
			t => tracing::warn!("no side effects implemented for accepting {t:?}"),
		}

		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn reject(&self, _uid: String, _activity: serde_json::Value) -> crate::Result<String> {
		todo!()
	}

	async fn undo(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		{
			let Some(old_aid) = activity.object().id() else {
				return Err(StatusCode::BAD_REQUEST.into());
			};
			let Some(old_activity) = model::activity::Entity::find_by_id(old_aid)
				.one(self.db()).await?
			else {
				return Err(StatusCode::NOT_FOUND.into());
			};
			if old_activity.actor != uid {
				return Err(StatusCode::FORBIDDEN.into());
			}
			match old_activity.activity_type {
				apb::ActivityType::Like => {
					model::like::Entity::delete(model::like::ActiveModel {
						actor: Set(old_activity.actor), likes: Set(old_activity.object.unwrap_or("".into())),
						..Default::default()
					}).exec(self.db()).await?;
				},
				apb::ActivityType::Follow => {
					model::relation::Entity::delete(model::relation::ActiveModel {
						follower: Set(old_activity.actor), following: Set(old_activity.object.unwrap_or("".into())),
						..Default::default()
					}).exec(self.db()).await?;
				},
				t => tracing::warn!("extra side effects for activity {t:?} not implemented"),
			}
		}
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}
}
