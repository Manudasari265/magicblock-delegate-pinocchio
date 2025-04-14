use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::find_program_address,
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::state::TokenAccount;
use pinocchio_token::instructions;

use crate::{
    error::MyProgramError,
    state::EscrowState
};

pub fn process_refund_offer_instruction (
    accounts: &[AccountInfo],
) -> ProgramResult {
    // # Accounts need to process take-offer - State & Data accounts
    // 
    // -> '[signer]' maker_acc - account requesting for the ecrow refund
    // -> '[]' maker_acc - creator of the escrow
    // -> '[]' mint_x - token offering by the maker
    // -> '[mut]' maker_ata_x - maker's ata account for token x 
    // -> '[mut]' vault - token account having tokens locked from maker
    // -> '[mut]' escrow_acc - state account storing the data 
    // -> '[]' - system_program - system program
    // -> '[]' - token_program - spl token program 
    let [maker_acc, mint_x, maker_ata_x, vault, escrow_acc, _system_program, _token_program, _remaining@ ..] = accounts 
      else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // check if the signer is the owner of the escrow account
    if !maker_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // check if the data is empty or not, if empty throw error
    if !escrow_acc.data_is_empty() {
        return Err(MyProgramError::AccountEmpty.into());
    }

    // access and extract the escrow account for escrow detaislss
    let escrow_data = escrow_acc
        .try_borrow_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    let escrow_account_details = bytemuck::try_from_bytes::<EscrowState>(&escrow_data)
        .map_err(|_| MyProgramError::DeserializationFailed)?;

    // check for only the signer's stored token and not both
    assert_eq!(escrow_account_details.mint_x, *mint_x.key());

    // now derive the vault account details similar to escrow
    let vault_acc_details = TokenAccount::from_account_info(vault)?;

    // prepare the seeds and re-derive the pda for signing opertations
    let seed = [(b"escrow"), maker_acc.key().as_slice(), &[escrow_account_details.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow_acc.key(), escrow_pda);

    let bump = [escrow_account_details.bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker_acc.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    instructions::Transfer {
        from: vault,
        to: maker_ata_x,
        authority: escrow_acc,
        amount: vault_acc_details.amount(),
    }
    .invoke_signed(
        &[seeds.clone()]
    )?;

    instructions::CloseAccount {
        account: vault,
        destination: maker_acc,
        authority: escrow_acc,
    }
    .invoke_signed(
        &[seeds]
    )?;
        
    unsafe {
        *maker_acc.borrow_mut_lamports_unchecked() += *escrow_acc.borrow_lamports_unchecked();
        *escrow_acc.borrow_mut_lamports_unchecked() = 0;
    };

    Ok(())
}