pub mod user;
pub mod object;
pub mod activity;
pub mod faker;

#[derive(Debug, Clone, thiserror::Error)]
#[error("missing required field: '{0}'")]
pub struct FieldError(pub &'static str);

	}
}
