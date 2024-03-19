#[derive(Debug, thiserror::Error)]
#[error("invalid type value")]
pub struct TypeValueError;

impl From<TypeValueError> for sea_orm::sea_query::ValueTypeErr {
	fn from(_: TypeValueError) -> Self {
		sea_orm::sea_query::ValueTypeErr
	}
}

impl From<TypeValueError> for sea_orm::TryGetError {
	fn from(_: TypeValueError) -> Self {
		sea_orm::TryGetError::Null("value is not a valid type".into())
	}
}

macro_rules! strenum {
	( $(pub enum $enum_name:ident { $($flat:ident),+ $($deep:ident($inner:ident)),*};)+ ) => {
		$(
			#[derive(PartialEq, Eq, Debug, Clone, Copy)]
			pub enum $enum_name {
				$($flat,)*
				$($deep($inner),)*
			}

			impl AsRef<str> for $enum_name {
				fn as_ref(&self) -> &str {
					match self {
						$(Self::$flat => stringify!($flat),)*
						$(Self::$deep(x) => x.as_ref(),)*
					}
				}
			}

			impl TryFrom<&str> for $enum_name {
				type Error = TypeValueError;

				fn try_from(value:&str) -> Result<Self, Self::Error> {
					match value {
						$(stringify!($flat) => Ok(Self::$flat),)*
						_ => {
							$(
								if let Ok(x) = $inner::try_from(value) {
									return Ok(Self::$deep(x));
								}
							)*
							Err(TypeValueError)
						},
					}
				}
			}

			impl From<$enum_name> for sea_orm::Value {
				fn from(value: $enum_name) -> sea_orm::Value {
					sea_orm::Value::String(Some(Box::new(value.as_ref().to_string())))
				}
			}

			impl sea_orm::sea_query::ValueType for $enum_name {
				fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
					match v {
						sea_orm::Value::String(Some(x)) =>
							Ok(<Self as TryFrom<&str>>::try_from(x.as_str())?),
						_ => Err(sea_orm::sea_query::ValueTypeErr),
					}
				}
			
				fn type_name() -> String {
					stringify!($enum_name).to_string()
				}
			
				fn array_type() -> sea_orm::sea_query::ArrayType {
					sea_orm::sea_query::ArrayType::String
				}
			
				fn column_type() -> sea_orm::sea_query::ColumnType {
					sea_orm::sea_query::ColumnType::String(Some(24))
				}
			}
			
			impl sea_orm::TryGetable for $enum_name {
				fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::prelude::QueryResult, index: I) -> Result<Self, sea_orm::TryGetError> {
					let x : String = res.try_get_by(index)?;
					Ok(Self::try_from(x.as_str())?)
				}
			}
		)*
	};
}

strenum! {
	pub enum BaseType {
		Invalid

		Object(ObjectType),
		Link(LinkType)
	};
	
	pub enum LinkType {
		Base,
		Mention
	};
	
	pub enum ObjectType {
		Object,
		Relationship,
		Tombstone

		Activity(ActivityType),
		Actor(ActorType),
		Collection(CollectionType),
		Status(StatusType)
	};
	
	pub enum ActorType {
		Application,
		Group,
		Organization,
		Person,
		Object
	};
	
	pub enum StatusType {
		Article,
		Event,
		Note,
		Place,
		Profile

		Document(DocumentType)
	};

	pub enum CollectionType {
		Collection,
		CollectionPage,
		OrderedCollection,
		OrderedCollectionPage
	};

	pub enum AcceptType {
		Accept,
		TentativeAccept
	};

	pub enum DocumentType {
		Document,
		Audio,
		Image,
		Page,
		Video
	};
	
	pub enum ActivityType {
		Activity,
		Add,
		Announce,
		Create,
		Delete,
		Dislike,
		Flag,
		Follow,
		Join,
		Leave,
		Like,
		Listen,
		Move,
		Read,
		Remove,
		Undo,
		Update,
		View

		IntransitiveActivity(IntransitiveActivityType),
		Accept(AcceptType),
		Ignore(IgnoreType),
		Offer(OfferType),
		Reject(RejectType)
	};
	
	pub enum IntransitiveActivityType {
		IntransitiveActivity,
		Arrive,
		Question,
		Travel
	};
	
	pub enum IgnoreType {
		Ignore,
		Block
	};
	
	pub enum OfferType {
		Offer,
		Invite
	};
	
	pub enum RejectType {
		Reject,
		TentativeReject
	};
}

#[cfg(test)]
mod test {
	#[test]
	fn assert_flat_types_serialize() {
		let x = super::IgnoreType::Block;
		assert_eq!("Block", <super::IgnoreType as AsRef<str>>::as_ref(&x));
	}

	#[test]
	fn assert_deep_types_serialize() {
		let x = super::StatusType::Document(super::DocumentType::Page);
		assert_eq!("Page", <super::StatusType as AsRef<str>>::as_ref(&x));
	}

	#[test]
	fn assert_flat_types_deserialize() {
		let x = super::ActorType::try_from("Person").expect("could not deserialize");
		assert_eq!(super::ActorType::Person, x);
	}

	#[test]
	fn assert_deep_types_deserialize() {
		let x = super::ActivityType::try_from("Invite").expect("could not deserialize");
		assert_eq!(super::ActivityType::Offer(super::OfferType::Invite), x);
	}
}
