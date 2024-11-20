#[derive(Debug, thiserror::Error)]
#[error("missing field '{0}'")]
pub struct FieldErr(pub &'static str);

pub type Field<T> = Result<T, FieldErr>;
