// TODO merge these flat maybe?
// but then db could theoretically hold an actor with type "Like" ... idk!
#[derive(Debug, Clone)]
pub enum Type {
	Object,
	ObjectType(ObjectType),
	Link,
	Mention, // TODO what about this???
	Activity,
	IntransitiveActivity,
	ActivityType(ActivityType),
	Collection,
	OrderedCollection,
	CollectionPage,
	OrderedCollectionPage,
	ActorType(ActorType),
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ObjectType(x) => write!(f, "{:?}", x),
			Self::ActivityType(x) => write!(f, "{:?}", x),
			Self::ActorType(x) => write!(f, "{:?}", x),
			_ => write!(f, "{:?}", self),
		}
	}
}

#[derive(sea_orm::EnumIter, sea_orm::DeriveActiveEnum, PartialEq, Eq, Debug, Clone, Copy)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ActivityType {
	Accept = 1,
	Add = 2,
	Announce = 3,
	Arrive = 4,
	Block = 5,
	Create = 6,
	Delete = 7,
	Dislike = 8,
	Flag = 9,
	Follow = 10,
	Ignore = 11,
	Invite = 12,
	Join = 13,
	Leave = 14,
	Like = 15,
	Listen = 16,
	Move = 17,
	Offer = 18,
	Question = 19,
	Reject = 20,
	Read = 21,
	Remove = 22,
	TentativeReject = 23,
	TentativeAccept = 24,
	Travel = 25,
	Undo = 26,
	Update = 27,
	View = 28,
}

#[derive(sea_orm::EnumIter, sea_orm::DeriveActiveEnum, PartialEq, Eq, Debug, Clone, Copy)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ActorType {
	Application = 1,
	Group = 2,
	Organization = 3,
	Person = 4,
	Service = 5,
}

#[derive(sea_orm::EnumIter, sea_orm::DeriveActiveEnum, PartialEq, Eq, Debug, Clone, Copy)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ObjectType {
	Article = 1,
	Audio = 2,
	Document = 3,
	Event = 4,
	Image = 5,
	Note = 6,
	Page = 7,
	Place = 8,
	Profile = 9,
	Relationship = 10,
	Tombstone = 11,
	Video = 12,
}
