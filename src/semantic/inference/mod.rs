pub(crate) mod constraints;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Int32,
    Bool,
    String,
    Placeholder(usize), // Unique type variable placeholder (e.g. T0, T1)
}
