use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

use crate::state::utils::DataLen;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EscrowState {
    pub maker: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub amount: [u8; 8],
    pub bump: u8,
}

impl DataLen for EscrowState {
    const LEN: usize = core::mem::size_of::<EscrowState>();
}

impl EscrowState {
    pub fn initialize(
        escrow_acc: &AccountInfo,
        maker: Pubkey,
        mint_x: Pubkey,
        mint_y: Pubkey,
        amount: [u8; 8],
        bump: u8,
    ) {
        let escrow =
            unsafe { &mut *(escrow_acc.borrow_mut_data_unchecked().as_ptr() as *mut Self) };

        escrow.maker = maker;
        escrow.mint_x = mint_x;
        escrow.mint_y = mint_y;
        escrow.amount = amount;
        escrow.bump = bump;
    }
}

 // load_mut_raw_unchecked
    // load_mut_safe_check
    // pub fn load_mut_raw_unchecked(account_info:: &[AccountInfo]) -> &mut Self {
    //     unsafe {
    //         &mut *{account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self}
    //     }
    // }

    // pub fn load_mut_checked(account_info: &[AccountInfo]) -> &mut Self {
    //     unsafe {
    //         assert_eq!(account_info.data_len(), EscrowState::LEN);
    //         assert_eq!(account_info.owner(), &crate::ID);
    //         &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self)
    //     }
    // }