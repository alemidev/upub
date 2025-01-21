use leptos_axum::LeptosRoutes;

impl super::WebRouter for axum::Router<upub::Context> {
	fn web_routes(self, ctx: &upub::Context) -> Self where Self: Sized {
		self.leptos_routes(
			ctx,
			leptos_axum::generate_route_list(upub_web::App),
			move || ""
		)
	}
}
