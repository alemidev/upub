use std::collections::{hash_map::Entry, HashMap};

use sea_orm::{ConnectionTrait, DbErr, EntityTrait, FromQueryResult, ModelTrait, QueryFilter};
use super::RichActivity;

#[allow(async_fn_in_trait)]
pub trait BatchFillable: Sized {
	async fn with_batched<E>(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>;
}


impl BatchFillable for Vec<RichActivity> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
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
				if let Some(v) = map.get(&object.internal) {
					// TODO wasteful because we clone every time, but we cant do remove otherwise multiple
					//      identical objects wont get filled (for example, a post boosted twice)
					element.accept(v.clone(), tx).await?;
				}
			}
		}
		Ok(self)
	}
}

impl BatchFillable for RichActivity {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: BatchFillableKey + Send + FromQueryResult + ModelTrait<Entity = E>,
		RichActivity: BatchFillableAcceptor<Vec<E::Model>>,
	{
		if let Some(ref obj) = self.object {
			let batch =E::find()
				.filter(E::comparison(vec![obj.internal]))
				.all(tx)
				.await?;
			self.accept(batch, tx).await?;
		}
		Ok(self)
	}
}


// welcome to interlocking trait hell, enjoy your stay
mod hell {
	use sea_orm::{sea_query::IntoCondition, ColumnTrait, ConnectionTrait, DbErr, EntityTrait};

use crate::selector::rich::{RichHashtag, RichMention};

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
	
#[allow(async_fn_in_trait)]
	pub trait BatchFillableAcceptor<B> {
		async fn accept(&mut self, batch: B, tx: &impl ConnectionTrait) -> Result<(), DbErr>;
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::attachment::Model>> for super::RichActivity {
		async fn accept(&mut self, batch: Vec<crate::model::attachment::Model>, _tx: &impl ConnectionTrait) -> Result<(), DbErr> {
			self.attachments = Some(batch);
			Ok(())
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::hashtag::Model>> for super::RichActivity {
		async fn accept(&mut self, batch: Vec<crate::model::hashtag::Model>, _tx: &impl ConnectionTrait) -> Result<(), DbErr> {
			self.hashtags = Some(batch.into_iter().map(|x| RichHashtag { hash: x }).collect());
			Ok(())
		}
	}
	
	impl BatchFillableAcceptor<Vec<crate::model::mention::Model>> for super::RichActivity {
		async fn accept(&mut self, batch: Vec<crate::model::mention::Model>, tx: &impl ConnectionTrait) -> Result<(), DbErr> {
			// TODO batch load users from mentions rather than doing for loop
			let mut mentions = Vec::new();
			for row in batch {
				// TODO filter only needed rows
				if let Some(user) = crate::model::actor::Entity::find_by_id(row.actor).one(tx).await? {
					mentions.push(RichMention {
						mention: row,
						fqn: format!("@{}@{}", user.preferred_username, user.domain),
						id: user.id,
					});
				}
			}
			self.mentions = Some(mentions);
			Ok(())
		}
	}
}

use hell::*;

