use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveValue::{NotSet, Set}, ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model};

#[axum::async_trait]
pub trait SideEffects {
	async fn process_like(&self, who: i64, what: i64, with: i64, when: chrono::DateTime<chrono::Utc>) -> crate::Result<()>;
	async fn process_undo(&self, who: i64, activity: impl apb::Activity) -> crate::Result<()>;
}

#[axum::async_trait]
impl SideEffects for super::Context {
	async fn process_like(&self, who: i64, what: i64, with: i64, when: chrono::DateTime<chrono::Utc>) -> crate::Result<()> {
		let like = model::like::ActiveModel {
			internal: NotSet,
			actor: Set(who),
			object: Set(what),
			activity: Set(with),
			published: Set(when),
		};
		model::like::Entity::insert(like).exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
			.filter(model::object::Column::Internal.eq(what))
			.exec(self.db())
			.await?;

		Ok(())
	}

	async fn process_undo(&self, who: i64, activity: impl apb::Activity) -> crate::Result<()> {
		let undone_object_id = activity.object().id().ok_or_else(UpubError::bad_request)?;
		match activity.activity_type() {
			Some(apb::ActivityType::Like) => {
				let internal_oid = model::object::Entity::ap_to_internal(&undone_object_id, self.db()).await?;
				model::like::Entity::delete_many()
					.filter(
						Condition::all()
							.add(model::like::Column::Actor.eq(who))
							.add(model::like::Column::Object.eq(internal_oid))
					)
					.exec(self.db())
					.await?;
				model::object::Entity::update_many()
					.filter(model::object::Column::Internal.eq(internal_oid))
					.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).sub(1))
					.exec(self.db())
					.await?;
			},
			Some(apb::ActivityType::Follow) => {
				let undone_aid = activity.object().id().ok_or_else(UpubError::bad_request)?;
				let internal_aid = model::activity::Entity::ap_to_internal(&undone_aid, self.db()).await?;
				model::relation::Entity::delete_many()
					.filter(model::relation::Column::Activity.eq(internal_aid))
					.exec(self.db())
					.await?;
				model::actor::Entity::update_many()
					.filter(model::actor::Column::Internal.eq(who))
					.col_expr(model::actor::Column::FollowingCount, Expr::col(model::actor::Column::FollowingCount).sub(1))
					.exec(self.db())
					.await?;
				model::actor::Entity::update_many()
					.filter(model::actor::Column::Id.eq(&undone_object_id))
					.col_expr(model::actor::Column::FollowersCount, Expr::col(model::actor::Column::FollowersCount).sub(1))
					.exec(self.db())
					.await?;
			},
			t => {
				tracing::error!("received 'Undo' for unimplemented activity type: {t:?}");
				return Err(StatusCode::NOT_IMPLEMENTED.into());
			},
		}
		

		Ok(())
	}
}
