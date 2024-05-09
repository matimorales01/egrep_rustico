use crate::{regex_rep::RegexRep, regex_value::RegexValue};

#[derive(Debug, Clone)]
pub struct RegexStep {
    pub val: RegexValue,
    pub rep: RegexRep,
}
