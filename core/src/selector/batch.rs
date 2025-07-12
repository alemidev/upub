use std::collections::{hash_map::Entry, HashMap};

use sea_orm::{ConnectionTrait, DbErr, EntityTrait, FromQueryResult, ModelTrait, QueryFilter};
use super::{RichActivity, RichObject, RichObjectOrActor};

#[allow(async_fn_in_trait)]
pub trait RichFillable: Sized {
	async fn load_batched_models(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>;
}

impl<T> RichFillable for T
where
	T: BatchFillable
{
	#[allow(clippy::needless_question_mark)]
	async fn load_batched_models(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr> {
		Ok(
			self
				.with_batched::<crate::model::attachment::Entity>(tx)
				.await?
				.with_batched::<crate::model::mention::Entity>(tx)
				.await?
				.with_batched::<crate::model::hashtag::Entity>(tx)
				.await?
				.with_batched::<crate::model::question_option::Entity>(tx)
				.await?
		)
	}
}


#[allow(async_fn_in_trait)]
pub trait BatchFillable: Sized {
	async fn with_batched<T>(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		T: BatchFillableComparison + EntityTrait,
		T::Model: Send + FromQueryResult + ModelTrait<Entity = T>,
		RichObject: BatchFillableLoader<T>;
}

impl BatchFillable for RichActivity {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
		RichObject: BatchFillableLoader<E>,
	{
		self.object = self.object.with_batched::<E>(tx).await?;
		Ok(self)
	}
}

impl BatchFillable for Vec<RichActivity> {
	async fn with_batched<E>(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
			E: BatchFillableComparison + EntityTrait,
			E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
			RichObject: BatchFillableLoader<E>
	{
		// TODO can we do this in-place rather than copying everything to a new vec?
		let mut out = Vec::new();
		for item in self {
			out.push(item.with_batched::<E>(tx).await?);
		}

		Ok(out)
	}
}


impl BatchFillable for Vec<RichObject> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
		RichObject: BatchFillableLoader<E>,
	{
		let ids : Vec<i64> = self.iter().filter_map(|x| Some(x.object.as_ref()?.internal)).collect();
		let batch_query = E::find()
			.filter(E::comparison(ids));
		let batch = RichObject::load(batch_query)
			.all(tx)
			.await?;
		let mut map : HashMap<i64, Vec<<RichObject as BatchFillableLoader<E>>::To>> = HashMap::new();
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

impl BatchFillable for RichObject {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
		RichObject: BatchFillableLoader<E>,
	{
		if let Some(ref obj) = self.object {
			let query = E::find()
				.filter(E::comparison(vec![obj.internal]));
			let batch = RichObject::load(query)
				.all(tx)
				.await?;
			self.accept(batch);
		}
		Ok(self)
	}
}

impl BatchFillable for RichObjectOrActor {
	async fn with_batched<E>(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
		E: BatchFillableComparison + EntityTrait,
		E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
		RichObject: BatchFillableLoader<E>,
	{
		self.object = self.object.with_batched::<E>(tx).await?;
		Ok(self)
	}
}

impl BatchFillable for Vec<RichObjectOrActor> {
	async fn with_batched<E>(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>
	where
			E: BatchFillableComparison + EntityTrait,
			E::Model: Send + FromQueryResult + ModelTrait<Entity = E>,
			RichObject: BatchFillableLoader<E>
	{
		// TODO can we do this in-place rather than copying everything to a new vec?
		let mut out = Vec::new();
		for item in self {
			out.push(item.with_batched::<E>(tx).await?);
		}

		Ok(out)
	}
}


// welcome to interlocking trait hell, enjoy your stay
mod hell {
	use sea_orm::{sea_query::IntoCondition, ColumnTrait, EntityName, Iden, Iterable, QuerySelect, RelationTrait, SelectColumns};

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

	impl BatchFillableComparison for crate::model::question_option::Entity {
		fn comparison(ids: Vec<i64>) -> sea_orm::Condition {
			crate::model::question_option::Column::Object.is_in(ids).into_condition()
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

	impl BatchFillableKey for crate::selector::RichMention {
		fn key(&self) -> i64 {
			self.mention.object
		}
	}

	impl BatchFillableKey for crate::model::hashtag::Model {
		fn key(&self) -> i64 {
			self.object
		}
	}

	impl BatchFillableKey for crate::selector::RichQuestionOption {
		fn key(&self) -> i64 {
			self.option.object
		}
	}
	
	pub trait BatchFillableLoader<From>
	where
		From : sea_orm::EntityTrait,
	{
		type To : sea_orm::FromQueryResult + BatchFillableKey;

		fn load(query: sea_orm::Select<From>) -> sea_orm::Selector<sea_orm::SelectModel<Self::To>> { query.into_model::<Self::To>() }
		fn accept(&mut self, batch: Vec<Self::To>);
	}
	
	impl BatchFillableLoader<crate::model::attachment::Entity> for super::RichObject {
		type To = crate::model::attachment::Model;
		fn accept(&mut self, batch: Vec<Self::To>) {
			self.attachments = Some(batch);
		}
	}
	
	impl BatchFillableLoader<crate::model::hashtag::Entity> for super::RichObject {
		type To = crate::model::hashtag::Model;
		fn accept(&mut self, batch: Vec<Self::To>) {
			self.hashtags = Some(batch);
		}
	}
	
	impl BatchFillableLoader<crate::model::mention::Entity> for super::RichObject {
		type To = crate::selector::RichMention;

		fn load(query: sea_orm::Select<crate::model::mention::Entity>) -> sea_orm::Selector<sea_orm::SelectModel<Self::To>> {
			let mut new_query = query
				.join(sea_orm::JoinType::LeftJoin, crate::model::mention::Relation::Actors.def())
				.select_only()
				.select_column_as(
					crate::model::actor::Column::PreferredUsername,
					format!("{}{}", crate::model::actor::Entity.table_name(), crate::model::actor::Column::PreferredUsername.to_string()),
				)
				.select_column_as(
					crate::model::actor::Column::Domain,
					format!("{}{}", crate::model::actor::Entity.table_name(), crate::model::actor::Column::Domain.to_string()),
				)
				.select_column_as(
					crate::model::actor::Column::Id,
					format!("{}{}", crate::model::actor::Entity.table_name(), crate::model::actor::Column::Id.to_string()),
				);

			for col in crate::model::mention::Column::iter() {
				new_query = new_query.select_column_as(
					col,
					format!("{}{}", crate::model::mention::Entity.table_name(), col.to_string())
				);
			}

			new_query
				.into_model::<crate::selector::RichMention>()
		}

		fn accept(&mut self, batch: Vec<Self::To>) {
			self.mentions = Some(batch);
		}
	}

	impl BatchFillableLoader<crate::model::question_option::Entity> for super::RichObject {
		type To = crate::selector::RichQuestionOption;

		fn load(query: sea_orm::Select<crate::model::question_option::Entity>) -> sea_orm::Selector<sea_orm::SelectModel<Self::To>> {
			let mut new_query = query
				.join(sea_orm::JoinType::LeftJoin, crate::model::question_option::Relation::QuestionAnswers.def())
				.select_only()
				.select_column_as(
					crate::model::question_answer::Column::Internal.sum(),
					format!(
						"{}{}",
						crate::model::question_option::Entity.table_name(),
						"votes",
					),
				);

			for col in crate::model::question_option::Column::iter() {
				new_query = new_query.select_column_as(
					col,
					format!("{}{}", crate::model::question_option::Entity.table_name(), col.to_string())
				);
			}

			new_query
				.group_by(crate::model::question_option::Column::Internal)
				.into_model::<crate::selector::RichQuestionOption>()
		}

		fn accept(&mut self, batch: Vec<Self::To>) {
			self.options = Some(batch);
		}
	}
}

use hell::*;

