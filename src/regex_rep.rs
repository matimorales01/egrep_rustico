/// Representa la repetición de una expresión regular.
#[derive(Debug, Clone, PartialEq)]
pub enum RegexRep {
    Any,
    Exact(usize),
    Range {
        min: Option<usize>,
        max: Option<usize>,
    },
}
