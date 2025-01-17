fn main() {
	console_error_panic_hook::set_once();

	tracing_subscriber::fmt()
		.with_writer(
			tracing_subscriber_wasm::MakeConsoleWriter::default()
				.map_trace_level_to(tracing::Level::DEBUG)
		)
		.with_ansi(false)
		.without_time()
		.init();

	leptos::mount::mount_to_body(upub_web::App);
}
