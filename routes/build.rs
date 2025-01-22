fn main() {
	println!("cargo::rerun-if-changed=../web/dist");
	#[cfg(feature = "web")]
	{
		println!("cargo::warning=searching frontend files in $WORKSPACE_ROOT/web/dist");
		let Ok(dist) = std::fs::read_dir(std::path::Path::new("../web/dist")) else {
			println!("cargo::error=could not find 'web/dist' dir: did you 'trunk build' the frontend crate?");
			return;
		};

		let mut found_wasm = false;
		let mut found_js = false;
		let mut found_index = false;
		let mut found_style = false;
		let mut found_favicon = false;
		let mut found_icon = false;
		let mut found_manifest = false;

		for f in dist.flatten() {
			if let Ok(ftype) = f.file_type() {
				if ftype.is_file() {
					let fname = f.file_name().to_string_lossy().to_string();
					if !found_wasm {
						found_wasm = if_matches_set_env_path("CARGO_UPUB_FRONTEND_WASM", &f, &fname, "upub-web", ".wasm");
					}
					if !found_js {
						found_js = if_matches_set_env_path("CARGO_UPUB_FRONTEND_JS", &f, &fname, "upub-web", ".js");
					}
					if !found_style {
						found_style = if_matches_set_env_path("CARGO_UPUB_FRONTEND_STYLE", &f, &fname, "style", ".css");
					}
					if !found_index {
						found_index = if_matches_set_env_path("CARGO_UPUB_FRONTEND_INDEX", &f, &fname, "index", ".html");
					}
					if !found_favicon {
						found_favicon = if_matches_set_env_path("CARGO_UPUB_FRONTEND_FAVICON", &f, &fname, "favicon", ".ico")
					}
					if !found_icon {
						found_icon = if_matches_set_env_path("CARGO_UPUB_FRONTEND_PWA_ICON", &f, &fname, "icon", ".png")
					}
					if !found_manifest {
						found_manifest = if_matches_set_env_path("CARGO_UPUB_FRONTEND_PWA_MANIFEST", &f, &fname, "manifest", ".json")
					}
				}
			}
		}

		if !found_wasm {
			println!("cargo::error=could not find wasm payload");
		}

		if !found_js {
			println!("cargo::error=could not find js bindings");
		}

		if !found_style {
			println!("cargo::error=could not find style sheet");
		}

		if !found_favicon {
			println!("cargo::error=could not find favicon image");
		}

		if !found_icon {
			println!("cargo::error=could not find pwa icon image");
		}

		if !found_manifest {
			println!("cargo::error=could not find pwa manifest");
		}

		if !found_index {
			println!("cargo::error=could not find html index");
		}
	}
}

fn if_matches_set_env_path(var: &str, f: &std::fs::DirEntry, fname: &str, first: &str, last: &str) -> bool {
	if fname.starts_with(first) && fname.ends_with(last) {
		match f.path().canonicalize() {
			Ok(path) => println!("cargo::rustc-env={var}={}", path.to_string_lossy()),
			Err(e) => println!("cargo::warning=could not canonicalize '{}': {e}", f.path().to_string_lossy()),
		}
		true
	} else {
		false
	}
}
