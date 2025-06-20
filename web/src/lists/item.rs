use leptos::prelude::*;
use crate::prelude::*;

use apb::{Base, Collection, Object};

#[component]
pub fn List(
	list: crate::Doc,
) -> impl IntoView {
	let lid = list.id().unwrap_or_default().to_string();
	// let author_id = list.attributed_to().id().ok().unwrap_or_default();
	let name = list.name().unwrap_or(lid.clone());
	let summary = list.summary().unwrap_or_default();
	view! {
		<div class="ml-3 mr-3 center">
			<a class="clean" href=Uri::web(U::List, &lid)><b class="big">{name}</b></a>
			<hr class="mb-0" />
			<p class="mt-0 mb-2"><small>{summary}</small></p>
		</div>
	}
}
