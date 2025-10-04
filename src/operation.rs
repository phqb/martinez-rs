#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    Intersection,
    Union,
    Difference,
    Xor,
}
