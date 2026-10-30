//! Optional mirroring of accepted lines into the git index ("stage what I reviewed").

mod index;
mod mode;
mod partial;

pub use index::sync_index;
pub use mode::StageMode;
pub use partial::partial_content;
