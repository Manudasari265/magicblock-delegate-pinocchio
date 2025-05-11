use pinocchio::{
    account_info::AccountInfo, 
    instruction::{Seed, Signer}, 
    program_error::ProgramError, 
    pubkey, 
    sysvars::{rent::Rent, Sysvar}, 
    ProgramResult
};

use crate::state::{
    utils::{get_seeds, deserialize_delegate_instruction_data},
};

use crate::types::DelegateAccountArgs;

pub const BUFFER: &[u8] = b"buffer";

pub fn process_delegate(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata, system_program, _rest @..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (seeds_data, config) = deserialize_delegate_instruction_data(instruction_data)?;

    let delegate_pda_seeds = seeds_data.clone();

    // get buffer seeds
    let buffer_seeds: &[&[u8]] = &[BUFFER, pda_acc.key().as_ref()];
    let pda_seeds: Vec<&[u8]> = seeds_data.iter()
        .map(|s| s.as_slice().collect());

    // find pdas
    let (_, delegate_account_bump) = pubkey::find_program_address(&pda_seeds, &crate::ID);
    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, &crate::ID);

    // get delegated pda signer seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let mut delegate_seeds = get_seeds(pda_seeds)?;

    // get buffer signer seeds
    let bump = [buffer_pda_bump];
    let seed_b = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&bump),
    ];

    let buffer_signer_seeds = Signer::from(&seed_b);

    // create buffer pda account
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: Rent::get()?.minimum_balance(pda_acc.data_len()),
        space: pda_acc.data_len() as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[buffer_signer_seeds.clone()])?;

    // prepare delegate args
    // struct DelegateConfig comes from instruction_data
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds: delegate_pda_seeds,
        validator: config.validator,
    };

    cpi_delegate(
        payer, 
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        delegate_signer_seeds,
    )?;

    close_pda_acc(payer, buffer_acc, system_program)?;

    
    Ok(())
}