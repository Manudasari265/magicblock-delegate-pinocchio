use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    account_info::{AccountInfo, Ref}, 
    cpi::invoke_signed, 
    instruction::{AccountMeta, Instruction, Seed, Signer}, 
    program_error::ProgramError,
    pubkey::{self, Pubkey}, 
    sysvars::{rent::Rent, Sysvar}, 
    ProgramResult
};
use pinocchio_log::log;

use crate::{
    state::{
        DataLen,
        EscrowState
    }
};

// delegation account public key from magicblock's ER sdk 
pub const DELEGATION_ACCOUNT: Pubkey = pinocchio_pubkey::pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

// config data fields needed for commiting back the state to the base layer
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct DelegateAccountConfig {
    /// how often (in milliseconds) the delegated account's state
    /// should be committed (synced) back to the base solana layer
    pub commit_frequency_ms: u32,
    /// it's a optional validator field which this account is bound to while
    /// designated and if set, onlt this validator can process transactions for the account
    pub validator: Option<Pubkey>,
    /// the seeds used to regenerate the pda and act as a signer via CPI.
    /// Needed to verify PDA authority during the delegation.
    pub seeds: Vec<Vec<u8>>,
}

impl Default for DelegateAccountConfig {
    fn default() -> Self {
        DelegateAccountConfig { 
            commit_frequency_ms: u32::MAX, 
            validator: None, 
            seeds: vec![],
        }
    }
}

pub fn process_delegate_account (
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [maker_acc, escrow_pda_delegate_acc, magicblock_acc, buffer_temp_acc, deelegation_record, delegation_metadata, system_program] = accounts
       else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // get the buffer seeds for finding pdas later and signing txns
    let buffer_seeds: &[&[u8]] = &[b"escrow", escrow_pda_delegate_acc.key().as_ref()];
    let escrow_seeds = &["escrow".as_bytes(), maker_acc.key().as_ref()];
    let delegation_record_seeds = &[b"delegation", deelegation_record.key().as_ref()];
    let delegation_meta_seeds = &[b"delegation_metadata", delegation_metadata.key().as_ref()];

    // only re-derive the pda account to be delegated and derive the pda for buffer account
    let (_, delegate_account_bump) = pubkey::find_program_address(
        escrow_seeds, 
        &crate::ID,
    );

    // derive the pda account for temporary state storage for buffer pda account bump
    let (_, buffer_pda_bump) = pubkey::find_program_address(
        buffer_seeds, 
        &crate::ID,
    );

    Ok(())
}

