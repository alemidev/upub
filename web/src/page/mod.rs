mod about;
pub use about::AboutPage;

mod config;
pub use config::ConfigPage;

mod debug;
pub use debug::DebugPage;

mod object;
pub use object::ObjectPage;

mod register;
pub use register::RegisterPage;

mod search;
pub use search::SearchPage;

mod timeline;
pub use timeline::TimelinePage;

mod actor;
pub use actor::view::UserPage;
pub use actor::follow::FollowPage;
