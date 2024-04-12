#[async_trait::async_trait]
pub trait Outbox {
	type Object: crate::Object;
	type Activity: crate::Activity;
	type Error: std::error::Error;

	async fn create_note(&self, uid: String, object: Self::Object) -> Result<String, Self::Error>;
	async fn create(&self, uid: String, activity: Self::Activity) -> Result<String, Self::Error>;
	async fn like(&self, uid: String, activity: Self::Activity) -> Result<String, Self::Error>;
	async fn follow(&self, uid: String, activity: Self::Activity) -> Result<String, Self::Error>;
	async fn accept(&self, uid: String, activity: Self::Activity) -> Result<String, Self::Error>;
	async fn reject(&self, _uid: String, _activity: Self::Activity) -> Result<String, Self::Error>;
	async fn undo(&self, uid: String, activity: Self::Activity) -> Result<String, Self::Error>;
}

#[async_trait::async_trait]
pub trait Inbox {
	type Activity: crate::Activity;
	type Error: std::error::Error;

	async fn create(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn like(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn follow(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn accept(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn reject(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn undo(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn delete(&self, activity: Self::Activity) -> Result<(), Self::Error>;
	async fn update(&self, activity: Self::Activity) -> Result<(), Self::Error>;
}