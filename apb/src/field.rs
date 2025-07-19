#[derive(Debug, thiserror::Error)]
#[error("missing field '{0}' in object '{1:?}'")]
pub struct FieldErr(pub &'static str, pub Option<String>);

pub type Field<T> = Result<T, FieldErr>;
