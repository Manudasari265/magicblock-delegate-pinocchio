use pinocchio::program_error::ProgramError;

pub mod make_offer;
pub mod refund_offer;
pub mod take_offer;
pub mod delegate;

pub use make_offer::*;
pub use refund_offer::*;
pub use take_offer::*;
pub use delegate::*;


#[repr(C)]
pub enum EscrowInstruction {
    MakeOffer = 0,
    TakeOffer = 1,
    RefundOffer = 2,
    Delegate = 3,
}

impl TryFrom<&u8> for EscrowInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(EscrowInstruction::MakeOffer),
            1 => Ok(EscrowInstruction::TakeOffer),
            2 => Ok(EscrowInstruction::RefundOffer),
            3 => Ok(EscrowInstruction::Delegate),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
