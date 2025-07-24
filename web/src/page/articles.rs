use leptos::prelude::*;
use crate::prelude::*;

#[component]
pub fn ArticlesPage() -> impl IntoView {
	view! {
		<div class="mt-3">
			<Loadable
				base=format!("{URL_BASE}/articles/page")
				element=move |obj| view! { <Item item=obj sep=true /> }
			/>
		</div>
	}
}
