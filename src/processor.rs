use crate::error::CrateError;
use crate::instruction::ContractInstruction;
use crate::utils::create_account;
use borsh::BorshDeserialize;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account;
/// Program state handler.
pub struct Processor {}

impl Processor {
    pub fn check_mint(
        pool_mint_info: &AccountInfo,
        pool_mint_authority_info: &AccountInfo,
    ) -> ProgramResult {
        let mint = if pool_mint_info.owner != &spl_token::id() {
            return Err(CrateError::IncorrectTokenProgramId.into());
        } else {
            spl_token::state::Mint::unpack(&pool_mint_info.data.borrow())
                .map_err(|_| CrateError::ExpectedMint)
        }?;

        if let COption::Some(ref pk) = mint.mint_authority {
            if pk != pool_mint_authority_info.key {
                return Err(CrateError::InvalidOwner.into());
            }
        } else {
            return Err(CrateError::InvalidOwner.into());
        };
        Ok(())
    }

    pub fn check_user_wallets(
        user_authority: &AccountInfo,
        user_wallet_x: &AccountInfo,
        user_wallet_y: &AccountInfo,
        pool_mint: &AccountInfo,
        amount: u64,
    ) -> ProgramResult {
        let user_wallet_x = spl_token::state::Account::unpack(&user_wallet_x.data.borrow())?;
        let user_wallet_y = spl_token::state::Account::unpack(&user_wallet_y.data.borrow())?;
        if &user_wallet_x.owner != user_authority.key || &user_wallet_y.owner != user_authority.key
        {
            return Err(CrateError::InvalidOwner.into());
        }
        if user_wallet_x.amount < amount {
            return Err(CrateError::NotEnoughTokens.into());
        }
        if &user_wallet_y.mint != pool_mint.key {
            return Err(CrateError::IncorrectPoolMint.into());
        }
        Ok(())
    }

    /// Issue a spl_token `Burn` instruction.
    pub fn token_burn<'a>(
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::burn(
            &spl_token::id(),
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[burn_account, mint, authority], signer_seeds)
    }

    /// Issue a spl_token `MintTo` instruction.
    pub fn token_mint_to<'a>(
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[mint, destination, authority], signers_seeds)
    }

    /// Issue a spl_token `Transfer` instruction.
    pub fn token_transfer<'a>(
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::transfer(
            &spl_token::id(),
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;
        invoke_signed(&ix, &[source, destination, authority], signers_seeds)
    }

    /// Issue a spl_token `InitializeAccount` instruction.
    pub fn initialize_account<'a>(
        account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        owner: &Pubkey,
        rent: AccountInfo<'a>,
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::initialize_account(
            &spl_token::id(),
            account.key,
            mint.key,
            owner,
        )?;
        invoke_signed(&ix, &[account, mint, rent], signers_seeds)
    }

    // pub fn init_account(program_info: AccountInfo, account_id: &Pubkey, mint_id: &Pubkey, authority_id: &Pubkey, bump_seed: u8) -> Result<(), ProgramError> {
    //     let authority_signature_seeds = [&authority_id[..32], &[bump_seed]];
    //     let signers = &[&authority_signature_seeds[..]];
    //     let ix = spl_token::instruction::initialize_account(program_id, account_id, mint_id, program_id)?;
    //     invoke_signed(&ix, &[], signers)
    // }

    pub fn change_x_to_y(
        program_id: &Pubkey,
        token_x_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user_wallets_authority_info = next_account_info(account_info_iter)?;
        let user_wallet_x_info = next_account_info(account_info_iter)?;
        let token_x_mint_info = next_account_info(account_info_iter)?;
        let user_wallet_y_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let pool_mint_authority_info = next_account_info(account_info_iter)?;
        let pool_wallet_x_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;
        let _system_program_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !user_wallets_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Self::check_mint(pool_mint_info, pool_mint_authority_info)?;
        Self::check_user_wallets(
            user_wallets_authority_info,
            user_wallet_x_info,
            user_wallet_y_info,
            pool_mint_info,
            token_x_amount,
        )?;

        if pool_wallet_x_info.owner != &solana_program::system_program::id()
            && pool_wallet_x_info.owner != &spl_token::id()
        {
            return Err(CrateError::AlreadyInUse.into());
        }

        let (pool_wallet_x_authority, bump_seed) =
            Pubkey::find_program_address(&[&user_wallet_x_info.key.to_bytes()], program_id);
        if *pool_wallet_x_info.key != pool_wallet_x_authority {
            msg!("Error: Associated address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        let signers_seeds = &[&user_wallet_x_info.key.to_bytes()[..32], &[bump_seed]];

        if pool_wallet_x_info.owner == &solana_program::system_program::id() {
            create_account::<Account>(
                user_wallets_authority_info.clone(),
                pool_wallet_x_info.clone(),
                &[signers_seeds],
                rent,
            )?;
            Self::initialize_account(
                pool_wallet_x_info.clone(),
                token_x_mint_info.clone(),
                &pool_wallet_x_authority,
                rent_info.clone(),
                &[signers_seeds],
            )?;
        } else {
            let account =
                spl_token::state::Account::unpack_unchecked(&pool_wallet_x_info.data.borrow())
                    .map_err(|_| CrateError::ExpectedAccount)?;
            if !account.is_initialized() {
                Self::initialize_account(
                    pool_wallet_x_info.clone(),
                    token_x_mint_info.clone(),
                    &pool_wallet_x_authority,
                    rent_info.clone(),
                    &[signers_seeds],
                )?;
            } else {
                if &account.mint != token_x_mint_info.key {
                    return Err(CrateError::IncorrectPoolMint.into());
                }
            }
        }

        Self::token_transfer(
            user_wallet_x_info.clone(),
            pool_wallet_x_info.clone(),
            user_wallets_authority_info.clone(),
            token_x_amount,
            &[],
        )?;

        let (pool_mint_authority, bump_seed) =
            Pubkey::find_program_address(&[&pool_mint_info.key.to_bytes()], program_id);
        if *pool_mint_authority_info.key != pool_mint_authority {
            return Err(CrateError::InvalidProgramAddress.into());
        }

        let signers_seeds = &[&pool_mint_info.key.to_bytes()[..32], &[bump_seed]];

        Self::token_mint_to(
            pool_mint_info.clone(),
            user_wallet_y_info.clone(),
            pool_mint_authority_info.clone(),
            token_x_amount,
            &[signers_seeds],
        )?;
        //
        //
        // let program_id = program_id_info.key;
        //
        // let user_wallet_x = spl_token::state::Account::unpack(&user_wallet_x_info.data.borrow())
        //     .map_err(|_| CrateError::ExpectedAccount)?;
        //
        // let mint_x_id = user_wallet_x.mint;
        //
        // let (token_y_id, bump_seed) =
        //     Pubkey::find_program_address(&[&mint_x_id[..]], program_id);
        //
        // if *authority_info.key != program_wallet_y {
        //     return Err(CrateError::InvalidProgramAddress.into());
        // }
        //
        // let token_y = Self::unpack_self_token_account(token_y_info, program_id)?;
        // let pool_mint = Self::unpack_mint(mint_y_info, program_id)?;

        Ok(())
    }

    pub fn change_y_to_x(
        program_id: &Pubkey,
        token_y_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user_wallets_authority_info = next_account_info(account_info_iter)?;
        let user_wallet_x_info = next_account_info(account_info_iter)?;
        let user_wallet_y_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let pool_wallet_x_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !user_wallets_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pool_wallet_x_authority, bump_seed) =
            Pubkey::find_program_address(&[&user_wallet_x_info.key.to_bytes()], program_id);
        if *pool_wallet_x_info.key != pool_wallet_x_authority {
            return Err(CrateError::InvalidProgramAddress.into());
        }

        let signers_seeds = &[&user_wallet_x_info.key.to_bytes()[..32], &[bump_seed]];

        Self::token_transfer(
            pool_wallet_x_info.clone(),
            user_wallet_x_info.clone(),
            pool_wallet_x_info.clone(),
            token_y_amount,
            &[signers_seeds],
        )?;

        Self::token_burn(
            user_wallet_y_info.clone(),
            pool_mint_info.clone(),
            user_wallets_authority_info.clone(),
            token_y_amount,
            &[],
        )?;
        Ok(())
    }

    /// Processes an instruction.
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = ContractInstruction::try_from_slice(input)?;
        match instruction {
            ContractInstruction::ChangeXtoY { amount } => {
                msg!("Instruction: ChangeXtoY");
                Self::change_x_to_y(program_id, amount, accounts)
            }
            ContractInstruction::ChangeYtoX { amount } => {
                msg!("Instruction: ChangeYtoX");
                Self::change_y_to_x(program_id, amount, accounts)
            }
        }
    }
}
