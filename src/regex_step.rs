use crate::{regex_rep::RegexRep, regex_value::RegexValue};
/// Representa un paso individual en una expresión regular.
#[derive(Debug, Clone)]
pub struct RegexStep {
    pub val: RegexValue,
    pub rep: RegexRep,
}
