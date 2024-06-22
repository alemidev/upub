use std::collections::{hash_map::Entry, HashMap};

use apb::{ActivityMut, LinkMut, ObjectMut};
use sea_orm::{sea_query::{IntoColumnRef, IntoCondition}, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityName, EntityTrait, FromQueryResult, Iden, Iterable, ModelTrait, Order, QueryFilter, QueryOrder, QueryResult, QuerySelect, RelationTrait, Select, SelectColumns};

use crate::model;

pub struct Query;

impl Query {
	pub fn activities(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			.distinct_on([
				(model::addressing::Entity, model::addressing::Column::Published).into_column_ref(),
				(model::activity::Entity, model::activity::Column::Internal).into_column_ref(),
			])
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Activities.def())
			.join(sea_orm::JoinType::LeftJoin, model::addressing::Relation::Objects.def())
			.filter(
				// TODO ghetto double inner join because i want to filter out tombstones
				Condition::any()
					.add(model::activity::Column::Id.is_not_null())
					.add(model::object::Column::Id.is_not_null())
			)
			.order_by(model::addressing::Column::Published, Order::Desc)
			.order_by(model::activity::Column::Internal, Order::Desc)
			.select_only();

		for col in model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::activity::Entity.table_name(), col.to_string()));
		}

		for col in model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
		}

		if let Some(uid) = my_id {
			select = select
				.join(
					sea_orm::JoinType::LeftJoin,
					model::object::Relation::Likes.def()
						.on_condition(move |_l, _r| model::like::Column::Actor.eq(uid).into_condition()),
				)
				.select_column_as(model::like::Column::Actor, format!("{}{}", model::like::Entity.table_name(), model::like::Column::Actor.to_string()));
		}

		select
	}

	pub fn objects(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			.distinct_on([
				(model::addressing::Entity, model::addressing::Column::Published).into_column_ref(),
				(model::object::Entity, model::object::Column::Internal).into_column_ref(),
			])
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Objects.def())
			.order_by(model::addressing::Column::Published, Order::Desc)
			.order_by(model::object::Column::Internal, Order::Desc)
			.select_only();

		for col in model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
		}

		if let Some(uid) = my_id {
			select = select
				.join(
					sea_orm::JoinType::LeftJoin,
					model::object::Relation::Likes.def()
						.on_condition(move |_l, _r| model::like::Column::Actor.eq(uid).into_condition()),
				)
				.select_column_as(model::like::Column::Actor, format!("{}{}", model::like::Entity.table_name(), model::like::Column::Actor.to_string()));
		}

		select
	}
}



pub struct RichActivity {
	pub activity: model::activity::Model,
	pub object: Option<model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<model::attachment::Model>>,
	pub hashtags: Option<Vec<model::hashtag::Model>>,
	pub mentions: Option<Vec<model::mention::Model>>,
}

impl FromQueryResult for RichActivity {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichActivity {
			activity: model::activity::Model::from_query_result(res, model::activity::Entity.table_name())?,
			object: model::object::Model::from_query_result(res, model::object::Entity.table_name()).ok(),
			liked: res.try_get(model::like::Entity.table_name(), &model::like::Column::Actor.to_string()).ok(),
			attachments: None, hashtags: None, mentions: None,
		})
	}
}

impl RichActivity {
	pub fn ap(self) -> serde_json::Value {
		let object = match self.object {
			None => apb::Node::maybe_link(self.activity.object.clone()),
			Some(o) => {
				// TODO can we avoid repeating this tags code?
				let mut tags = Vec::new();
				if let Some(mentions) = self.mentions {
					for mention in mentions {
						tags.push(
							apb::new()
								.set_link_type(Some(apb::LinkType::Mention))
								.set_href(&mention.actor)
								// TODO do i need to set name? i could join while batch loading or put the @name in
								// each mention object...
						);
					}
				}
				if let Some(hashtags) = self.hashtags {
					for hash in hashtags {
						tags.push(
							// TODO ewwww set_name clash and cant use builder, wtf is this
							LinkMut::set_name(apb::new(), Some(&format!("#{}", hash.name)))
								.set_link_type(Some(apb::LinkType::Hashtag))
								// TODO do we need to set href too? we can't access context here, quite an issue!
						);
					}
				}
				apb::Node::object(
					o.ap()
						.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
						.set_tag(apb::Node::array(tags))
						.set_attachment(match self.attachments {
							None => apb::Node::Empty,
							Some(vec) => apb::Node::array(
								vec.into_iter().map(|x| x.ap()).collect()
							),
						})
				)
			},
		};
		self.activity.ap().set_object(object)
	}
}

pub struct RichObject {
	pub object: model::object::Model,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<model::attachment::Model>>,
	pub hashtags: Option<Vec<model::hashtag::Model>>,
	pub mentions: Option<Vec<model::mention::Model>>,
}

impl FromQueryResult for RichObject {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichObject {
			object: model::object::Model::from_query_result(res, model::object::Entity.table_name())?,
			liked: res.try_get(model::like::Entity.table_name(), &model::like::Column::Actor.to_string()).ok(),
			attachments: None, hashtags: None, mentions: None,
		})
	}
}

impl RichObject {
	pub fn ap(self) -> serde_json::Value {
		// TODO can we avoid repeating this tags code?
		let mut tags = Vec::new();
		if let Some(mentions) = self.mentions {
			for mention in mentions {
				tags.push(
					apb::new()
						.set_link_type(Some(apb::LinkType::Mention))
						.set_href(&mention.actor)
						// TODO do i need to set name? i could join while batch loading or put the @name in
						// each mention object...
				);
			}
		}
		if let Some(hashtags) = self.hashtags {
			for hash in hashtags {
				tags.push(
					// TODO ewwww set_name clash and cant use builder, wtf is this
					LinkMut::set_name(apb::new(), Some(&format!("#{}", hash.name)))
						.set_link_type(Some(apb::LinkType::Hashtag))
						// TODO do we need to set href too? we can't access context here, quite an issue!
				);
			}
		}
		self.object.ap()
			.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
			.set_tag(apb::Node::array(tags))
			.set_attachment(match self.attachments {
				None => apb::Node::Empty,
				Some(vec) => apb::Node::array(
					vec.into_iter().map(|x| x.ap()).collect()
				)
			})
	}
}

#[async_trait::async_trait]
pub trait BatchFillable: Sized {
	async fn with_batched<E>(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
		RichObject: BatchFillableAcceptor<Vec<E::Model>>;
}

#[async_trait::async_trait]
impl BatchFillable for Vec<RichActivity> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
		RichObject: BatchFillableAcceptor<Vec<E::Model>>
	{
		let ids : Vec<i64> = self.iter().filter_map(|x| Some(x.object.as_ref()?.internal)).collect();
		let batch = E::find()
			.filter(E::comparison(ids))
			.all(tx)
			.await?;
		let mut map : HashMap<i64, Vec<E::Model>> = HashMap::new();
		for element in batch {
			match map.entry(element.key()) {
				Entry::Occupied(mut x) => { x.get_mut().push(element); },
				Entry::Vacant(x) => { x.insert(vec![element]); },
			}
		}
		for element in self.iter_mut() {
			if let Some(ref object) = element.object {
				if let Some(v) = map.remove(&object.internal) {
					element.accept(v);
				}
			}
		}
		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for Vec<RichObject> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
		RichObject: BatchFillableAcceptor<Vec<E::Model>>
	{
		let ids : Vec<i64> = self.iter().map(|x| x.object.internal).collect();
		let batch = E::find()
			.filter(E::comparison(ids))
			.all(tx)
			.await?;
		let mut map : HashMap<i64, Vec<E::Model>> = HashMap::new();
		for element in batch {
			match map.entry(element.key()) {
				Entry::Occupied(mut x) => { x.get_mut().push(element); },
				Entry::Vacant(x) => { x.insert(vec![element]); },
			}
		}
		for element in self.iter_mut() {
			if let Some(v) = map.remove(&element.object.internal) {
				element.accept(v);
			}
		}
		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for RichActivity {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
		RichObject: BatchFillableAcceptor<Vec<E::Model>>
	{
		if let Some(ref obj) = self.object {
			let batch =E::find()
				.filter(E::comparison(vec![obj.internal]))
				.all(tx)
				.await?;
			self.accept(batch);
		}
		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for RichObject {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
		RichObject: BatchFillableAcceptor<Vec<E::Model>>
	{
		let batch = E::find()
			.filter(E::comparison(vec![self.object.internal]))
			.all(tx)
			.await?;
		self.accept(batch);
		Ok(self)
	}
}


// welcome to interlocking trait hell, enjoy your stay
mod hell {
	use sea_orm::{sea_query::IntoCondition, ColumnTrait};

	pub trait BatchFillableComparison {
		fn comparison(ids: Vec<i64>) -> sea_orm::Condition;
	}

	impl BatchFillableComparison for crate::model::attachment::Entity {
		fn comparison(ids: Vec<i64>) -> sea_orm::Condition {
			crate::model::attachment::Column::Object.is_in(ids).into_condition()
		}
	}

	impl BatchFillableComparison for crate::model::mention::Entity {
		fn comparison(ids: Vec<i64>) -> sea_orm::Condition {
			crate::model::mention::Column::Object.is_in(ids).into_condition()
		}
	}

	impl BatchFillableComparison for crate::model::hashtag::Entity {
		fn comparison(ids: Vec<i64>) -> sea_orm::Condition {
			crate::model::hashtag::Column::Object.is_in(ids).into_condition()
		}
	}
	
	pub trait BatchFillableKey {
		fn key(&self) -> i64;
	}

	impl BatchFillableKey for crate::model::attachment::Model {
		fn key(&self) -> i64 {
			self.object
		}
	}

	impl BatchFillableKey for crate::model::mention::Model {
		fn key(&self) -> i64 {
			self.object
		}
	}

	impl BatchFillableKey for crate::model::hashtag::Model {
		fn key(&self) -> i64 {
			self.object
		}
	}
	
	pub trait BatchFillableAcceptor<B> {
		fn accept(&mut self, batch: B);
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::attachment::Model>> for super::RichActivity {
		fn accept(&mut self, batch: Vec<crate::model::attachment::Model>) {
			self.attachments = Some(batch);
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::hashtag::Model>> for super::RichActivity {
		fn accept(&mut self, batch: Vec<crate::model::hashtag::Model>) {
			self.hashtags = Some(batch);
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::mention::Model>> for super::RichActivity {
		fn accept(&mut self, batch: Vec<crate::model::mention::Model>) {
			self.mentions = Some(batch);
		}
	}

	impl BatchFillableAcceptor<Vec<crate::model::attachment::Model>> for super::RichObject {
		fn accept(&mut self, batch: Vec<crate::model::attachment::Model>) {
			self.attachments = Some(batch);
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::hashtag::Model>> for super::RichObject {
		fn accept(&mut self, batch: Vec<crate::model::hashtag::Model>) {
			self.hashtags = Some(batch);
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::mention::Model>> for super::RichObject {
		fn accept(&mut self, batch: Vec<crate::model::mention::Model>) {
			self.mentions = Some(batch);
		}
	}
}
use hell::*;
