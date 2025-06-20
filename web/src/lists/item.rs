use leptos::{either::Either, prelude::*};
use crate::prelude::*;

use apb::{Base, Object};

#[component]
pub fn List(
	list: crate::Doc,
	#[prop(default = true)] controls: bool,
) -> impl IntoView {
	let lid = list.id().unwrap_or_default().to_string();
	// let author_id = list.attributed_to().id().ok().unwrap_or_default();
	let name = list.name().unwrap_or(lid.clone());
	let summary = list.summary().unwrap_or_default();
	view! {
		<div class="ml-1 mr-1">
			<a class="clean" href=Uri::web(U::List, &lid)><code class="cw center color">{name}</code></a>
			<p class="center mt-0 mb-1"><small>{summary}</small></p>
		</div>
	}
}
