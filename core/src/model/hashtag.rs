use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "hashtags")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub object: i64,
	pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Objects,
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl crate::ext::IntoActivityPub for Model {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		use apb::LinkMut;
		apb::new()
			.set_name(Some(format!("#{}", self.name)))
			.set_link_type(Some(apb::LinkType::Hashtag))
			.set_href(Some(crate::url!(ctx, "/tags/{}", self.name)))
	}
}
