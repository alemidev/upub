fn main() {
	_ = console_log::init_with_level(log::Level::Info);
	console_error_panic_hook::set_once();

	leptos::mount_to_body(upub_web::App);
}
