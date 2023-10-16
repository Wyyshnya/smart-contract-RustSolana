use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    sysvar::Sysvar,
    rent::Rent,
};

use solana_program::program_pack::IsInitialized;
use spl_token::state::Account;
use spl_token::state::AccountState::{Initialized, Uninitialized};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data[0] {
        0 => initialize_store(accounts, instruction_data),
        1 => update_price(accounts, instruction_data),
        2 => sell(accounts, instruction_data),
        3 => buy(accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
fn initialize_store(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let store_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter);

    let mut store_data = store_account.try_borrow_mut_data()?;
    let mut token_data = token_account.try_borrow_mut_data()?;
    let mut owner_data = owner_account.try_borrow_mut_data()?;

    let store = &mut Account::unpack_unchecked(&mut store_data)?;
    if store.state == Uninitialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let owner_pubkey = owner_account.unwrap().key;
    let price = 1000; // Set the initial price here

    store.state = Initialized;
    store.mint = *token_account.key;
    store.owner = *owner_pubkey;
    store.amount = price;

    Account::pack(*store, &mut store_data)?;

    msg!("Store initialized successfully!");

    Ok(())
}

fn update_price(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() != 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let accounts_iter = &mut accounts.iter();

    let store_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;

    let mut store_data = store_account.try_borrow_mut_data()?;
    let store = &mut Account::unpack_unchecked(&mut store_data)?;

    if store.owner != *owner_account.key {
        return Err(ProgramError::Custom(1)); // Unauthorized owner
    }

    let new_price = u64::from_le_bytes(data.try_into().unwrap());
    store.amount = new_price;

    Account::pack(* store, &mut store_data)?;

    msg!("Price updated successfully!");

    Ok(())
}

fn sell(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() != 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let accounts_iter = &mut accounts.iter();

    let store_account = next_account_info(accounts_iter)?;
    let seller_token_account = next_account_info(accounts_iter)?;
    let buyer_token_account = next_account_info(accounts_iter)?;
    let seller_owner_account = next_account_info(accounts_iter)?;

    let mut store_data = store_account.try_borrow_mut_data()?;
    let mut seller_token_data = seller_token_account.try_borrow_mut_data()?;
    let mut buyer_token_data = buyer_token_account.try_borrow_mut_data()?;

    let store = &mut Account::unpack_unchecked(&mut store_data)?;

    if store.amount == 0 {
        return Err(ProgramError::Custom(2)); // Price not set
    }

    if seller_token_account.owner != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if store.mint != *seller_token_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    if *store_account.key != *seller_owner_account.key {
        return Err(ProgramError::Custom(1)); // Unauthorized seller
    }

    let price = u64::from_le_bytes(data.try_into().unwrap());
    if price < store.amount {
        return Err(ProgramError::Custom(3)); // Insufficient payment
    }

    // Transfer tokens from the seller to the buyer
    spl_token::instruction::transfer(
        &spl_token::id(),
        seller_token_account.key,
        buyer_token_account.key,
        seller_owner_account.key,
        &[],
        price,
    )?;

    msg!("Tokens sold successfully!");

    Ok(())
}

fn buy(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() != 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let accounts_iter = &mut accounts.iter();

    let store_account = next_account_info(accounts_iter)?;
    let seller_token_account = next_account_info(accounts_iter)?;
    let buyer_token_account = next_account_info(accounts_iter)?;
    let buyer_owner_account = next_account_info(accounts_iter);

    let mut store_data = store_account.try_borrow_mut_data()?;
    let mut seller_token_data = seller_token_account.try_borrow_mut_data()?;
    let mut buyer_token_data = buyer_token_account.try_borrow_mut_data()?;

    let store = &mut Account::unpack_unchecked(&mut store_data)?;

    if store.amount == 0 {
        return Err(ProgramError::Custom(2)); // Price not set
    }

    if seller_token_account.owner != spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if store.mint != *seller_token_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let buyer_owner_pubkey = buyer_owner_account.unwrap().key;

    if *buyer_owner_pubkey != store.owner {
        return Err(ProgramError::Custom(1)); // Unauthorized buyer
    }

    let price = u64::from_le_bytes(data.try_into().unwrap());
    if price < store.amount {
        return Err(ProgramError::Custom(3)); // Insufficient payment
    }

    // Transfer SOL from the buyer to the seller
    let system_program = next_account_info(accounts_iter)?;
    let transfer_to_system_ix = solana_program::system_instruction::transfer(
        buyer_owner_pubkey,
        seller_token_account.key,
        price,
    );
    let acc= [buyer_owner_account.clone(), seller_token_account.clone(), system_program.clone()].iter().map(|ac| ac.as_ref().unwrap()).collect();
    solana_program::program::invoke(
        &transfer_to_system_ix,
        acc,
    )?;

    msg!("Tokens bought successfully!");

    Ok(())
}

entrypoint!(process_instruction);


