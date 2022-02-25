use crate::find_program_address;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::sysvar;

/// Instructions supported by the program
#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum ContractInstruction {
    /// Accounts:
    /// [RS] User transfer authority
    /// [W] Source account ( token X)
    /// [R] Token X mint
    /// [W] Destination account (pool token)
    /// [W] Pool mint
    /// [R] Pool authority
    /// [W] Pool wallet (token X)
    /// [R] Rent sysvar
    /// [R] System program
    /// [R] Token program id
    ChangeXtoY { amount: u64 },
    /// Accounts:
    /// [RS] User transfer authority
    /// [W] Destination account ( token X)
    /// [W] Source account (pool token)
    /// [W] Pool mint
    /// [W] Pool wallet (token X)
    /// [R] Token program id
    ChangeYtoX { amount: u64 },
}

pub fn change_x_to_y(
    program_id: &Pubkey,
    user_wallets_authority_id: &Pubkey,
    user_wallet_x_id: &Pubkey,
    token_x_mint_id: &Pubkey,
    user_wallet_y_id: &Pubkey,
    pool_mint_id: &Pubkey,
    amount: u64,
) -> Instruction {
    let (pool_wallet_x_id, _) = find_program_address(program_id, user_wallet_x_id);
    let (pool_mint_authority_id, _) = find_program_address(program_id, pool_mint_id);
    let accounts = vec![
        AccountMeta::new_readonly(*user_wallets_authority_id, true),
        AccountMeta::new(*user_wallet_x_id, false),
        AccountMeta::new(*token_x_mint_id, false),
        AccountMeta::new(*user_wallet_y_id, false),
        AccountMeta::new(*pool_mint_id, false),
        AccountMeta::new_readonly(pool_mint_authority_id, false),
        AccountMeta::new(pool_wallet_x_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &ContractInstruction::ChangeXtoY { amount },
        accounts,
    )
}

pub fn change_y_to_x(
    program_id: &Pubkey,
    user_wallets_authority_id: &Pubkey,
    user_wallet_x_id: &Pubkey,
    user_wallet_y_id: &Pubkey,
    pool_mint_id: &Pubkey,
    amount: u64,
) -> Instruction {
    let (pool_wallet_x_id, _) = find_program_address(program_id, user_wallet_x_id);

    let accounts = vec![
        AccountMeta::new_readonly(*user_wallets_authority_id, true),
        AccountMeta::new(*user_wallet_x_id, false),
        AccountMeta::new(*user_wallet_y_id, false),
        AccountMeta::new(*pool_mint_id, false),
        AccountMeta::new(pool_wallet_x_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &ContractInstruction::ChangeYtoX { amount },
        accounts,
    )
}
