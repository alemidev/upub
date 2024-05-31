pub mod config;
pub mod errors;
pub mod server;
pub mod model;
pub mod ext;

pub use server::Context;
pub use config::Config;
pub use errors::UpubResult as Result;
pub use errors::UpubError as Error;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
