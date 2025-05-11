use pinocchio::program_error::ProgramError;

#[derive(Debug, Clone, PartialEq)]
pub enum MyProgramError {
    // unable to deserialize
    DeserializationFailed,
    // overflow error
    WriteOverflow,
    // invalid instruction data
    InvalidInstructionData,
    // pda mismatch
    PdaMismatch,
    // Invalid Owner
    InvalidOwner,
    // Account is Empty
    AccountEmpty,
    // Failed serialization
    SerializationFailed,
}

impl From<MyProgramError> for ProgramError {
    fn from(e: MyProgramError) -> Self {
        Self::Custom(e as u32)
    }
}