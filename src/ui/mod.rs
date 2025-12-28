//! UI components module.
//!
//! Contains ratatui widgets for displaying the application interface.

pub mod filters;
pub mod list;
pub mod search;
pub mod tabs;

pub use filters::render_filters;
pub use list::render_list;
pub use search::render_search;
pub use tabs::render_tabs;
