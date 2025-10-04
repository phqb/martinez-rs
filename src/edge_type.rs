#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(crate) enum EdgeType {
    Normal,
    NonContributing,
    SameTransition,
    DifferentTransition,
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::Normal
    }
}
