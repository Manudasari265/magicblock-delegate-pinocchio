// # Import the necessary crates and states
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    instruction::{Seed, Signer},
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_token::state::TokenAccount;
use pinocchio_token::instructions;

use crate::{
    error::MyProgramError,
    state::EscrowState
};

// # Instruction steps to process take-offer - Logic
// 
// -> extract the required accounts needed for the take-offer
// -> check or validate by having fail-safe checks
// -> extract the on-chain state by bytecasting it to define type struct
// -> derive the data from the token account for escrow amount which is the vault account here
// -> prepare the seeds for re-deriving the escrow-pda to match if it's the correct escrow-account
// -> make the 1st transfer from taker_ata_y to maker_ata_y
// -> maker 2nd second transfer from vault to taker_ata_a
// -> finally close the account by maker_acc(user)


pub fn process_take_offer_instruction(
    accounts: &[AccountInfo],
) -> ProgramResult {
    // # Accounts need to process take-offer - State & Data accounts
    // 
    // -> '[signer]' taker_acc - account sending & receiving the escrow
    // -> '[]' maker_acc - creator of the escrow
    // -> '[]' mint_x - token offering by the maker
    // -> '[]' mint_y - token receiving to the maker
    // -> '[mut]' maker_ata_y - maker's ata account for token y 
    // -> '[mut]' taker_ata_x - taker's ata account for token x
    // -> '[mut]' taker_ata_y -  taker's ata account for token y
    // -> '[mut]' vault - token account having tokens locked from maker
    // -> '[mut]' escrow_acc - state account storing the data 
    // -> '[]' - system_program - system program
    // -> '[]' - token_program - spl token program 
    let [taker_acc, maker_acc, mint_x, mint_y, taker_ata_x, taker_ata_y, maker_ata_y, vault, escrow_acc, _system_program, _token_program, _remaining@ ..] = accounts 
      else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let escrow_acc_data = escrow_acc
        .try_borrow_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;
    
    let escrow_account = bytemuck::try_from_bytes::<EscrowState>(&escrow_acc_data)
        .map_err(|_| MyProgramError::DeserializationFailed)?;
    let vault_acc_details = TokenAccount::from_account_info(vault)?;

    assert_eq!(escrow_account.mint_x, *mint_x.key());
    assert_eq!(escrow_account.mint_y, *mint_y.key());

    //? preparing the seeds from the maker from vault to re-derive the escrow-account
    let seed = [(b"escrow"), maker_acc.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..]; //? you get pda & bump
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow_acc.key(), escrow_pda);

    instructions::Transfer {
        from: taker_ata_y,
        to: maker_ata_y,
        authority: taker_acc,
        amount: u64::from_le_bytes(escrow_account.amount),
    }
    .invoke()?;

    let bump = [escrow_account.bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker_acc.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    instructions::Transfer {
        from: vault,
        to: taker_ata_x,
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
        *escrow_acc.borrow_mut_lamports_unchecked() = 0
    };


    Ok(())
}