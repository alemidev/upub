#[derive(Debug, thiserror::Error)]
#[error("missing field '{0}'")]
pub struct FieldErr(pub &'static str);

pub type Field<T> = Result<T, FieldErr>;


// TODO this trait is really ad-hoc and has awful naming...

pub trait OptionalString {
	fn str(self) -> Option<String>;
}

impl OptionalString for Field<&str> {
	fn str(self) -> Option<String> {
		self.ok().map(|x| x.to_string())
	}
}
