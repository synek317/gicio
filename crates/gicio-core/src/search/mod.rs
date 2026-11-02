//! Text search across the project's files, like "Find in files" in an editor.

mod file_filter;
mod line_hit;
mod matcher;
mod query;
mod results;
mod run;

pub use line_hit::LineHit;
pub use query::{SearchPattern, SearchQuery};
pub use results::{FileHits, SearchResults};
pub use run::search_project;
