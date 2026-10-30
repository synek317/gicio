mod compute;
mod load;
mod model;
mod ops;

pub use compute::compute_hunks;
pub use load::{file_texts, load_diff, FileTexts};
pub use model::{DiffLine, FileDiff, FileStatus, Hunk, LineKind, Side};
pub use ops::{diff_ops, split_lines, DiffOptions};
