use axum::extract::{Multipart, Path, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use upub::Context;

use crate::AuthIdentity;

pub async fn upload(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	mut multipart: Multipart,
) -> crate::ApiResult<()> {
	if !auth.is_local() {
		return Err(crate::ApiError::forbidden());
	}

	let mut uploaded_something = false;
	while let Some(field) = multipart
		.next_field()
		.await
		.unwrap() // TODO OOOPS THIS SLIPPED GET RID OF IT
	{
		let _ = if let Some(filename) = field.file_name() {
			filename.to_string()
		} else {
			tracing::warn!("skipping multipart field {field:?}");
			continue;
		};

		let data = match field.bytes().await {
			Ok(x) => x,
			Err(e) => {
				tracing::error!("error reading multipart part: {e:?}");
				continue;
			},
		};

		let name = sha256::digest(data.as_ref());
		let path = format!("{}{name}", ctx.cfg().files.path);

		tokio::fs::File::open(path).await?.write_all(&data).await?;
		uploaded_something = true;
	}

	if uploaded_something {
		Ok(())
	} else {
		Err(crate::ApiError::bad_request())
	}
}

pub async fn download(
	State(ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Path(id): Path<String>,
) -> crate::ApiResult<Vec<u8>> {
	let path = format!("{}{id}", ctx.cfg().files.path);
	let mut buffer = Vec::new();
	tokio::fs::File::open(path)
		.await?
		.read_to_end(&mut buffer)
		.await?;

	Ok(buffer)
}
