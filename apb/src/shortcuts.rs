use crate::{Collection, Object};


pub trait Shortcuts: crate::Object {
	fn likes_count(&self) -> crate::Field<i32> {
		let x = self
			.likes()
			.inner()?
			.total_items()?
			.min(i32::MAX as u64)
			as i32;
		Ok(x)
	}

	fn shares_count(&self) -> crate::Field<i32> {
		let x = self
			.shares()
			.inner()?
			.total_items()?
			.min(i32::MAX as u64)
			as i32;
		Ok(x)
	}

	fn replies_count(&self) -> crate::Field<i32> {
		let x = self
			.replies()
			.inner()?
			.total_items()?
			.min(i32::MAX as u64)
			as i32;
		Ok(x)
	}

	fn image_url(&self) -> crate::Field<String> {
		let image_node = self.image();
		let image  = image_node.inner()?;
		let url = image.url();
		let id = url.id()?;
		Ok(id.to_string())
	}

	fn icon_url(&self) -> crate::Field<String> {
		let icon_node = self.icon();
		let icon  = icon_node.inner()?;
		let url = icon.url();
		let id = url.id()?;
		Ok(id.to_string())
	}
}

impl<T: crate::Object> Shortcuts for T {}
