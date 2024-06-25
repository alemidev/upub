use std::collections::{hash_map::Entry, HashMap};

use sea_orm::{ConnectionTrait, DbErr, EntityTrait, FromQueryResult, ModelTrait, QueryFilter};
use super::{RichActivity, RichObject};

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

