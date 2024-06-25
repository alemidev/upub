use apb::{ActivityMut, BaseMut, ObjectMut};
use sea_orm::{ActiveValue::{NotSet, Set}, DbErr, EntityTrait};
use upub::traits::fetch::PullError;

#[derive(Debug, Clone, clap::Subcommand)]
/// available actions to take on relays
pub enum RelayCommand {
	/// get all current pending and accepted relays
	Status,
	/// request to follow a specific relay
	Follow {
		/// relay actor to follow (must be full AP id, like for pleroma)
		actor: String,
	},
	/// accept a pending relay request
	Accept {
		/// relay actor to accept (must be full AP id, like for pleroma)
		actor: String,
	},
	/// retract a follow relation to a relay, stopping receiving content
	Unfollow {
		/// relay actor to unfollow (must be full AP id, like for pleroma)
		actor: String,
	},
	/// remove a follow relation from a relay, stopping sending content
	Remove {
		/// relay actor to unfollow (must be full AP id, like for pleroma)
		actor: String,
	},
}

pub async fn relay(ctx: upub::Context, action: RelayCommand) -> Result<(), PullError> {
	match action {
		RelayCommand::Status => {
			let internal_actor = upub::model::actor::Entity::ap_to_internal(ctx.base(), ctx.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(ctx.base().to_string()))?;

			tracing::info!("active sinks:");
			for sink in upub::Query::related(None, Some(internal_actor), false)
				.into_model::<upub::model::actor::Model>()
				.all(ctx.db())
				.await?
			{
				tracing::info!("[>>] {} {}", sink.name.unwrap_or_default(), sink.id);
			}

			tracing::info!("active sources:");
			for source in  upub::Query::related(Some(internal_actor), None, false)
				.into_model::<upub::model::actor::Model>()
				.all(ctx.db())
				.await?
			{
				tracing::info!("[<<] {} {}", source.name.unwrap_or_default(), source.id);
			}
		},

		RelayCommand::Follow { actor } => {
			let aid = ctx.aid(&upub::Context::new_id());
			let payload = apb::new()
				.set_id(Some(&aid))
				.set_activity_type(Some(apb::ActivityType::Follow))
				.set_actor(apb::Node::link(ctx.base().to_string()))
				.set_object(apb::Node::link(actor.clone()))
				.set_to(apb::Node::links(vec![actor.clone()]))
				.set_cc(apb::Node::links(vec![apb::target::PUBLIC.to_string()]))
				.set_published(Some(chrono::Utc::now()));
			let job = upub::model::job::ActiveModel {
				internal: NotSet,
				activity: Set(aid.clone()),
				job_type: Set(upub::model::job::JobType::Outbound),
				actor: Set(ctx.base().to_string()),
				target: Set(None),
				payload: Set(Some(payload)),
				attempt: Set(0),
				published: Set(chrono::Utc::now()),
				not_before: Set(chrono::Utc::now()),
			};
			tracing::info!("following relay {actor}");
			upub::model::job::Entity::insert(job).exec(ctx.db()).await?;
		},

		RelayCommand::Accept { actor } => {
			let their_internal = upub::model::actor::Entity::ap_to_internal(&actor, ctx.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(actor.clone()))?;
			let my_internal = upub::model::actor::Entity::ap_to_internal(ctx.base(), ctx.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(ctx.base().to_string()))?;
			let relation = upub::Query::related(Some(their_internal), Some(my_internal), true)
				.into_model::<upub::model::relation::Model>()
				.one(ctx.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(format!("relation-{their_internal}-{my_internal}")))?;
			let activity = upub::model::activity::Entity::find_by_id(relation.activity)
				.one(ctx.db())
				.await?
				.ok_or_else(|| DbErr::RecordNotFound(format!("activity#{}", relation.activity)))?;
			let aid = ctx.aid(&upub::Context::new_id());
			let payload = apb::new()
				.set_id(Some(&aid))
				.set_activity_type(Some(apb::ActivityType::Accept(apb::AcceptType::Accept)))
				.set_actor(apb::Node::link(ctx.base().to_string()))
				.set_object(apb::Node::link(activity.id))
				.set_to(apb::Node::links(vec![actor.clone()]))
				.set_cc(apb::Node::links(vec![apb::target::PUBLIC.to_string()]))
				.set_published(Some(chrono::Utc::now()));
			let job = upub::model::job::ActiveModel {
				internal: NotSet,
				activity: Set(aid.clone()),
				job_type: Set(upub::model::job::JobType::Outbound),
				actor: Set(ctx.base().to_string()),
				target: Set(None),
				payload: Set(Some(payload)),
				attempt: Set(0),
				published: Set(chrono::Utc::now()),
				not_before: Set(chrono::Utc::now()),
			};
			tracing::info!("accepting relay {actor}");
			upub::model::job::Entity::insert(job).exec(ctx.db()).await?;
		},

		RelayCommand::Remove { .. } => todo!(),

		RelayCommand::Unfollow { .. } => todo!(),
	}

	Ok(())
}
