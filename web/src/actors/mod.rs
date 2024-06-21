pub mod follow;
pub mod posts;
pub mod header;
pub mod activity;

use leptos_router::Params; // TODO can i remove this?
#[derive(Clone, leptos::Params, PartialEq)]
struct IdParam {
	id: Option<String>,
}
