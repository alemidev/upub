use crate::{activitypub::PUBLIC_TARGET, model::{config, credential}};
use super::{activity, object, user, Audience};
use sea_orm::IntoActiveModel;

pub async fn faker(db: &sea_orm::DatabaseConnection, domain: String, count: u64) -> Result<(), sea_orm::DbErr> {
	use sea_orm::{EntityTrait, Set};

	let root = super::user::Model {
		id: format!("{domain}/users/root"),
		name: Some("μpub".into()),
		domain: crate::activitypub::domain(&domain),
		preferred_username: "root".to_string(),
		summary: Some("hello world! i'm manually generated but served dynamically from db! check progress at https://git.alemi.dev/upub.git".to_string()),
		following: None,
		following_count: 0,
		followers: None,
		followers_count: 0,
		icon: Some("https://cdn.alemi.dev/social/circle-square.png".to_string()),
		image: Some("https://cdn.alemi.dev/social/someriver-xs.jpg".to_string()),
		inbox: None,
		shared_inbox: None,
		outbox: None,
		actor_type: crate::activitystream::object::actor::ActorType::Person,
		created: chrono::Utc::now(),
		updated: chrono::Utc::now(),
		private_key: None,
		// TODO generate a fresh one every time
		public_key: "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA01IkF1A8t7zUS7x3ZqIx
NvG/X43JEJTdgPOFufOKMBTK48uMqrODK3wflaK6cSOMWkoOYHyWNhw10vL53GR9
iOOGUHTaU7aoV7KBb5Fbh/tN9ZyN/S6FdCfpXze93CzpKQGNJa4SZ4FAFw2Ji1JJ
pdT7r+vaa5TkYTQuIgeHFisVEQAhmlz9xPbfPRPxSskroK7OnJSLMZ+aJvJHc9uX
kOq9wKgagV/I5ka7H3rRLTG1mBC97lrSfNdXfPRF//S4mFs095s+Pbnj0Hajf+qf
1Dryarmn6EmQi+diyhxFD3E8PWtMSBHrK+nmsiE58zjR/gd7G8SrXr4jtW+jcI6n
UQIDAQAB
-----END PUBLIC KEY-----".into()
	};

	user::Entity::insert(root.clone().into_active_model()).exec(db).await?;

	config::Entity::insert(config::ActiveModel {
		id: Set(root.id.clone()),
		accept_follow_requests: Set(true),
		show_followers: Set(true),
		show_following: Set(true),
		show_following_count: Set(true),
		show_followers_count: Set(true),
	}).exec(db).await?;

	credential::Entity::insert(credential::ActiveModel {
		id: Set(root.id.clone()),
		email: Set("mail@example.net".to_string()),
		password: Set(sha256::digest("very-strong-password")),
	}).exec(db).await?;

	let context = uuid::Uuid::new_v4().to_string();

	for i in (0..count).rev() {
		let oid = uuid::Uuid::new_v4();
		let aid = uuid::Uuid::new_v4();
		object::Entity::insert(object::ActiveModel {
			id: Set(format!("{domain}/objects/{oid}")),
			name: Set(None),
			object_type: Set(crate::activitystream::object::ObjectType::Note),
			attributed_to: Set(Some(format!("{domain}/users/root"))),
			summary: Set(None),
			context: Set(Some(context.clone())),
			content: Set(Some(format!("[{i}] Tic(k). Quasiparticle of intensive multiplicity. Tics (or ticks) are intrinsically several components of autonomously numbering anorganic populations, propagating by contagion between segmentary divisions in the order of nature. Ticks - as nonqualitative differentially-decomposable counting marks - each designate a multitude comprehended as a singular variation in tic(k)-density."))),
			published: Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
			comments: Set(0),
			likes: Set(0),
			shares: Set(0),
			to: Set(Audience(vec![PUBLIC_TARGET.to_string()])),
			bto: Set(Audience::default()),
			cc: Set(Audience(vec![])),
			bcc: Set(Audience::default()),
		}).exec(db).await?;

		activity::Entity::insert(activity::ActiveModel {
			id: Set(format!("{domain}/activities/{aid}")),
			activity_type: Set(crate::activitystream::object::activity::ActivityType::Create),
			actor: Set(format!("{domain}/users/root")),
			object: Set(Some(format!("{domain}/objects/{oid}"))),
			target: Set(None),
			published: Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
			to: Set(Audience(vec![PUBLIC_TARGET.to_string()])),
			bto: Set(Audience::default()),
			cc: Set(Audience(vec![])),
			bcc: Set(Audience::default()),
		}).exec(db).await?;
	}

	Ok(())
}
