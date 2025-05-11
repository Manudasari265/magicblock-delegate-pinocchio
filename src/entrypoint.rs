use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::instruction::{self, DelegationProgram};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the default panic handler.
default_panic_handler!();

#[inline(always)]
pub fn process_instruction(
    _program_id: &Pubkey, 
    accounts: &[AccountInfo],
    instruction_data: &[u8], 
) -> ProgramResult {
    let (discriminator_variant, instruction_data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

    match DelegationProgram::try_from(instruction_data)? {
        DelegationProgram::Delegate => instruction::process_delegate(accounts, instruction_data)?,
    }

    Ok(())
}
