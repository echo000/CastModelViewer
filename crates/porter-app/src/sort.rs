/// Sorting state for asset columns.
#[derive(Default, Debug, Clone, Copy)]
pub enum Sort {
    /// Not currently sorted.
    #[default]
    None,
    /// Sorted in ascending order.
    Ascending,
    /// Sorted in descending order.
    Descending,
}
