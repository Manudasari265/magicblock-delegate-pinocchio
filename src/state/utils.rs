use bytemuck::from_bytes;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::PUBKEY_BYTES,
};

use crate::{
    consts::DELEGATION_PROGRAM_ID,
    error::MyProgramError,
    types::{DelegateAccountArgs, DelegateConfig},
};

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

pub fn deserialize_delegate_instruction_data(instruction_data: &[u8]) -> Result<(Vec<Vec<u8>>, DelegateConfig), ProgramError> {
    let mut offset = 0;

    // first byte provides total number of seeds
    if instruction_data.len() < 1 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let num_seeds = instruction_data[0] as usize;
    offset += 1;

    // extract the delegated seeds
    let mut seeds = Vec::with_capacity(num_seeds);

    for _ in 0..num_seeds {
        if instruction_data.len() < offset + 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // first byte is out seed length
        let seed_len = instruction_data[0] as usize;
        offset += 1;

        let seed = instruction_data[offset..offset + seed_len].to_vec();
        seeds.push(seed);
        offset += seed_len;
    }

    // bytemuck deserialization DelegateConfig
    let config = parse_delegate_config(&instruction_data[offset..])?;

    Ok((seeds, config))
}

pub fn parse_delegate_config(instruction_data: &[u8]) -> Result<DelegateConfig, ProgramError> {
    if instruction_data.len() < 4 {
        return Err(MyProgramError::SerializationFailed.into());
    }

    let commit_frequency_ms = *from_bytes::u32(&instruction_data[..4]);

    let validator = if instruction_data.len() >= 36 {
        Some(instruction_data[4..36].try_into().unwrap())
    } else {
        None
    };

    Ok(DelegateConfig {
        commit_frequency_ms,
        validator,
    })
}

#[inline(always)]
pub fn get_seeds<'a>(seeds_vec: Vec<&'a [u8]>) -> Result<Vec<Seed<'a>>, ProgramError> {
    let mut seeds: Vec<Seed<'a>> = Vec::with_capacity(seeds_vec.len() + 1);

    // Add the regular seeds from the provided slice
    for seed in seeds_vec {
        seeds.push(Seed::from(seed));
    }

    Ok(seeds)
}

pub fn cpi_delegate(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    owner_program: &AccountInfo,
    buffer_acc: &AccountInfo,
    delegation_record: &AccountInfo,
    delegation_metadata: &AccountInfo,
    system_program: &AccountInfo,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    let account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::readonly(owner_program.key()),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let data: Vec<u8> = serialize_delegate_account_args(&delegate_args);

    //call Instruction
    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: &account_metas,
        data: &data,
    };

    let acc_infos = [
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[signer_seeds])?;
    Ok(())
}

pub fn close_pda_acc(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    // Step 1 - Lamports to zero
    unsafe {
        *payer.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }

    // Step 2 - Empty the data
    pda_acc.realloc(0, false).unwrap();

    // Step 3 - Send to System Program
    unsafe { pda_acc.assign(system_program.key()) };

    Ok(())
}

#[inline(always)]
pub fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_acc_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

#[inline(always)]
pub fn load_acc_mut<T: DataLen + Initialized>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    load_acc_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub fn load_acc_mut_unchecked<T: DataLen>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut T) })
}

#[inline(always)]
pub fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(MyProgramError::InvalidInstructionData.into());
    }
    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

pub fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(data as *const T as *const u8, T::LEN) }
}

pub fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN) }
}