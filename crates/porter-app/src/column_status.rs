use crate::Sort;

/// A columns sort status.
#[derive(Debug, Clone, Copy)]
pub struct ColumnStatus {
    pub index: usize,
    pub sort: Sort,
}

impl ColumnStatus {
    /// Construct a new column status.
    pub const fn new(index: usize, sort: Sort) -> Self {
        Self { index, sort }
    }
}
