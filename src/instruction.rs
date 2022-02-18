use crate::error::CrateError;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeXtoY {
    /// Token amount to deposit
    pub token_x_amount: u64,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeYtoX {
    /// Token amount to withdraw
    pub token_y_amount: u64,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum ContractInstruction {
    ChangeXtoY(ChangeXtoY),
    ChangeYtoX(ChangeYtoX),
}

impl ContractInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, res) = input
            .split_first()
            .ok_or_else(|| CrateError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                unimplemented!("reserve for init instruction")
            }
            1 => {
                let (token_x_amount, _) = Self::unpack_u64(res)?;
                Self::ChangeXtoY(ChangeXtoY { token_x_amount })
            }
            2 => {
                let (token_y_amount, _) = Self::unpack_u64(res)?;
                Self::ChangeYtoX(ChangeYtoX { token_y_amount })
            }
            _ => return Err(CrateError::InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (amount, rest) = input.split_at(8);
            let amount = amount
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(CrateError::InvalidInstruction)?;
            Ok((amount, rest))
        } else {
            Err(CrateError::InvalidInstruction.into())
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match &*self {
            Self::ChangeXtoY(ChangeXtoY { token_x_amount }) => {
                buf.push(1);
                buf.extend_from_slice(&token_x_amount.to_le_bytes())
            }
            Self::ChangeYtoX(ChangeYtoX { token_y_amount }) => {
                buf.push(2);
                buf.extend_from_slice(&token_y_amount.to_le_bytes())
            }
        }
        buf
    }

    pub fn change_x_to_y(
        program_id: &Pubkey,
        token_program_id: &Pubkey,
        user_wallets_authority_id: &Pubkey,
        user_wallet_x_id: &Pubkey,
        user_wallet_y_id: &Pubkey,
        pool_contract_id: &Pubkey,
        pool_mint_id: &Pubkey,
        pool_wallet_x_id: &Pubkey,
        authority_id: &Pubkey,
        token_x_amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let instr_data = ContractInstruction::ChangeXtoY(ChangeXtoY { token_x_amount });
        let data = instr_data.pack();

        let accounts = vec![
            AccountMeta::new_readonly(*user_wallets_authority_id, true),
            AccountMeta::new(*user_wallet_x_id, false),
            AccountMeta::new(*user_wallet_y_id, false),
            AccountMeta::new_readonly(*pool_contract_id, false),
            AccountMeta::new(*pool_mint_id, false),
            AccountMeta::new(*pool_wallet_x_id, false),
            AccountMeta::new_readonly(*authority_id, false),
            AccountMeta::new_readonly(*token_program_id, false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }

    pub fn change_y_to_x(
        program_id: &Pubkey,
        token_program_id: &Pubkey,
        user_wallets_authority_id: &Pubkey,
        user_wallet_x_id: &Pubkey,
        user_wallet_y_id: &Pubkey,
        pool_contract_id: &Pubkey,
        pool_mint_id: &Pubkey,
        pool_wallet_x_id: &Pubkey,
        authority_id: &Pubkey,
        token_y_amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let instr_data = ContractInstruction::ChangeYtoX(ChangeYtoX { token_y_amount });
        let data = instr_data.pack();

        let accounts = vec![
            AccountMeta::new_readonly(*user_wallets_authority_id, true),
            AccountMeta::new(*user_wallet_x_id, false),
            AccountMeta::new(*user_wallet_y_id, false),
            AccountMeta::new_readonly(*pool_contract_id, false),
            AccountMeta::new(*pool_mint_id, false),
            AccountMeta::new(*pool_wallet_x_id, false),
            AccountMeta::new_readonly(*authority_id, false),
            AccountMeta::new_readonly(*token_program_id, false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }
}
