use super::Slot;

#[derive(Debug)]
pub enum JumpCondition {
    Unconditional,
    IfZero(Slot),
    IfNotZero(Slot),
}

impl std::fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JumpCondition::Unconditional => write!(f, "always"),
            JumpCondition::IfZero(slot) => write!(f, "if {} == 0", slot),
            JumpCondition::IfNotZero(slot) => write!(f, "if {} != 0", slot),
        }
    }
}
