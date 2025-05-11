use pinocchio::program_error::ProgramError;

pub mod delegate;

pub use delegate::*;

#[repr(u8)]
pub enum DelegationProgram {
    Delegate,
    Undelegate,
    CommitAccounts,
    CommitAndUndelegateAccounts,
}

impl TryFrom<&u8> for DelegationProgram {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(DelegationProgram::Delegate),
            1 => Ok(DelegationProgram::Undelegate),
            2 => Ok(DelegationProgram::CommitAccounts),
            3 => Ok(DelegationProgram::CommitAndUndelegateAccounts),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}