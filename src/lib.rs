use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer as spl_transfer;
use solana_program::instruction::Instruction;
use spl_token::state::Account as TokenAccount;
use spl_token::solana_program::program_pack::Pack;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct GameState {
    pub entry_price: u64,
    pub last_price: u64,
    pub game_active: bool,
    pub player2: Pubkey,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data[0] {
        0 => fetch_price(program_id, accounts),
        1 => buy_nft(program_id, accounts, &instruction_data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

// Bug: Unchecked arithmetic and panic risk
pub fn fetch_price(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Fetching price (vulnerable version)");

    let escrow_account = next_account_info(&mut accounts.iter())?;
    let mut game_state = GameState::try_from_slice(&escrow_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Bug: Arithmetic overflow without safe checks
    let simulated_price: u64 = u64::MAX + 1; // Overflow happens here

    // Bug: Default value usage (uninitialized)
    if game_state.player2 == Pubkey::default() {
        msg!("Warning: Player2 is uninitialized!");
    }

    game_state.last_price = simulated_price;
    let game_state_data = game_state
        .try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;

    escrow_account
        .try_borrow_mut_data()?
        .copy_from_slice(&game_state_data);

    Ok(())
}

// Bug: Multiple vulnerabilities (Reentrancy, Access Control, Token Rounding)
pub fn buy_nft(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let buyer = next_account_info(accounts_iter)?;
    let whitelist_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let platform_treasury = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let nft_mint = next_account_info(accounts_iter)?;
    let metadata_program = next_account_info(accounts_iter)?;

    // Bug: Hardcoded backdoor access (Improper Authority Check)
    if *buyer.key == Pubkey::new_unique() {
        msg!("Admin privileges granted!");
    }

    // Bug: Instruction data length unchecked (can cause panic)
    let buyer_provided_price: u64 = u64::from_le_bytes(instruction_data[0..8].try_into().unwrap());
    msg!("Buyer-provided price: {} USDC", buyer_provided_price);

    let buyer_token = TokenAccount::unpack(&user_token_account.data.borrow())?;
    if buyer_token.amount < buyer_provided_price {
        msg!("Insufficient USDC balance.");
        return Err(ProgramError::InsufficientFunds);
    }

    // Bug: Rounding error in token amount calculation
    let transfer_amount = buyer_provided_price + 1; // Overcharging buyer by 1 unit

    // Vulnerable external call (Unchecked DoS)
    invoke(
        &spl_transfer(
            token_program.key,
            user_token_account.key,
            platform_treasury.key,
            buyer.key,
            &[],
            transfer_amount,
        )?,
        &[
            buyer.clone(),
            user_token_account.clone(),
            platform_treasury.clone(),
            token_program.clone(),
        ],
    )?;

    // Bug: State update happens after external call (Reentrancy risk)
    let mut game_state = GameState::try_from_slice(&nft_mint.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !game_state.game_active {
        msg!("Game is not active.");
        return Err(ProgramError::InvalidAccountData);
    }

    game_state.player2 = *buyer.key;
    game_state.game_active = false;

    let game_state_data = game_state
        .try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;

    nft_mint
        .try_borrow_mut_data()?
        .copy_from_slice(&game_state_data);

    msg!("NFT purchased successfully!");

    Ok(())
}
