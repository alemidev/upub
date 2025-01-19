use leptos::prelude::*;
use crate::{prelude::*, timeline::any::Loadable, FALLBACK_IMAGE_URL};

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
