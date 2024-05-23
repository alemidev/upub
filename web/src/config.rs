
#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct Config {
	#[serde(default)]
	pub filters: FiltersConfig,

	#[serde_inline_default(true)]
	pub collapse_content_warnings: bool,

	#[serde_inline_default(true)]
	pub loop_videos: bool,

	#[serde_inline_default(true)]
	pub infinite_scroll: bool,

	#[serde_inline_default("#BF616A".to_string())]
	pub accent_color: String,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_default::DefaultFromSerde)]
pub struct FiltersConfig {
	#[serde_inline_default(false)]
	pub replies: bool,

	#[serde_inline_default(false)]
	pub likes: bool,

	#[serde_inline_default(true)]
	pub creates: bool,

	#[serde_inline_default(true)]
	pub announces: bool,

	#[serde_inline_default(true)]
	pub follows: bool,

	#[serde_inline_default(true)]
	pub orphans: bool,
}

impl FiltersConfig {
	pub fn visible(&self, object_type: apb::ObjectType) -> bool {
		match object_type {
			apb::ObjectType::Note | apb::ObjectType::Document(_) => self.orphans,
			apb::ObjectType::Activity(apb::ActivityType::Like | apb::ActivityType::EmojiReact) => self.likes,
			apb::ObjectType::Activity(apb::ActivityType::Create) => self.creates,
			apb::ObjectType::Activity(apb::ActivityType::Announce) => self.announces,
			apb::ObjectType::Activity(
				apb::ActivityType::Follow | apb::ActivityType::Accept(_) | apb::ActivityType::Reject(_)
			) => self.follows,
			_ => true,
		}
	}
}
