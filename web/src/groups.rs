use leptos::prelude::*;
use crate::prelude::*;

#[component]
pub fn GroupList() -> impl IntoView {
	view! {
		<div>
			<Loadable
				base=format!("{URL_BASE}/groups/page")
				convert=U::Actor
				element=|obj| view! { <ActorBanner object=obj /><hr/> }
			/>
		</div>
	}
}
