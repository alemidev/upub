fn main() {
	#[cfg(all(feature = "web", feature = "web-build-fe"))]
	{
		println!("cargo:warning=running sub-process to build frontend");
		let status = std::process::Command::new("cargo")
			.current_dir("web")
			.args(["build", "--profile=wasm-release", "--target=wasm32-unknown-unknown"])
			.status()
			.unwrap();
		assert!(status.success(), "failed building wasm bundle");
	}
}
