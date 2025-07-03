use super::Slot;

#[derive(Debug)]
pub enum JumpCondition {
    Unconditional,
    Zero(Slot),
    NotZero(Slot),
    Greater(Slot, Slot),
    GreaterOrEqual(Slot, Slot),
    Less(Slot, Slot),
    LessOrEqual(Slot, Slot),
    Equal(Slot, Slot),
    NotEqual(Slot, Slot),
}

impl std::fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JumpCondition::Unconditional => write!(f, "always"),
            JumpCondition::Zero(slot) => write!(f, "{} == 0", slot),
            JumpCondition::NotZero(slot) => write!(f, "{} != 0", slot),
            JumpCondition::Equal(lhs, rhs) => write!(f, "{} == {}", lhs, rhs),
            JumpCondition::NotEqual(lhs, rhs) => write!(f, "{} != {}", lhs, rhs),
            JumpCondition::Greater(lhs, rhs) => write!(f, "{} > {}", lhs, rhs),
            JumpCondition::GreaterOrEqual(lhs, rhs) => write!(f, "{} >= {}", lhs, rhs),
            JumpCondition::Less(lhs, rhs) => write!(f, "{} < {}", lhs, rhs),
            JumpCondition::LessOrEqual(lhs, rhs) => write!(f, "{} <= {}", lhs, rhs),
        }
    }
}
