use crate::error::CrateError;
use crate::instruction::{ChangeXtoY, ChangeYtoX, ContractInstruction};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use std::cmp::min;

/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Unpacks a spl_token `Account`.
    pub fn unpack_self_token_account(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Account, CrateError> {
        if account_info.owner != token_program_id {
            Err(CrateError::IncorrectTokenProgramId)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
                .map_err(|_| CrateError::ExpectedAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    pub fn unpack_mint(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Mint, CrateError> {
        if account_info.owner != token_program_id {
            Err(CrateError::IncorrectTokenProgramId)
        } else {
            spl_token::state::Mint::unpack(&account_info.data.borrow())
                .map_err(|_| CrateError::ExpectedMint)
        }
    }

    /// Calculates the authority id by generating a program address.
    pub fn authority_id(
        program_id: &Pubkey,
        my_info: &Pubkey,
        bump_seed: u8,
    ) -> Result<Pubkey, CrateError> {
        Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[bump_seed]], program_id)
            .or(Err(CrateError::InvalidProgramAddress))
    }

    /// Issue a spl_token `Burn` instruction.
    pub fn token_burn<'a>(
        token_program: AccountInfo<'a>,
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        bump_seed: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let token_program_bytes = token_program.key.to_bytes();
        let authority_signature_seeds = [&token_program_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = spl_token::instruction::burn(
            token_program.key,
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(
            &ix,
            &[burn_account, mint, authority, token_program],
            signers,
        )
    }

    /// Issue a spl_token `MintTo` instruction.
    pub fn token_mint_to<'a>(
        token_program: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let token_program_bytes = token_program.key.to_bytes();
        let authority_signature_seeds = [&token_program_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[mint, destination, authority, token_program], signers)
    }

    /// Issue a spl_token `Transfer` instruction.
    pub fn token_transfer<'a>(
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let token_program_bytes = token_program.key.to_bytes();
        let authority_signature_seeds = [&token_program_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;
        invoke_signed(
            &ix,
            &[source, destination, authority, token_program],
            signers,
        )
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
        dbg!(accounts.len());
        // let program_token = next_account_info(account_info_iter)?;
        let user_wallets_authority_info = next_account_info(account_info_iter)?;
        let user_wallet_x_info = next_account_info(account_info_iter)?;
        let user_wallet_y_info = next_account_info(account_info_iter)?;
        let pool_contract_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let pool_wallet_x_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;

        let program_id = pool_contract_info.owner;
        let (program_authority_id, bump_seed) =
            Pubkey::find_program_address(&[&user_wallet_x_info.key.to_bytes()], program_id);
        if *authority_info.key != program_authority_id {
            return Err(CrateError::InvalidProgramAddress.into());
        }

        Self::token_transfer(
            token_program_info.clone(),
            user_wallet_x_info.clone(),
            pool_wallet_x_info.clone(),
            user_wallets_authority_info.clone(),
            token_x_amount,
            bump_seed,
        )?;
        Self::token_mint_to(
            token_program_info.clone(),
            pool_mint_info.clone(),
            user_wallet_y_info.clone(),
            authority_info.clone(),
            token_x_amount,
            bump_seed,
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
        Ok(())
    }

    /// Processes an instruction.
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = ContractInstruction::unpack(input)?;
        match instruction {
            ContractInstruction::ChangeXtoY(ChangeXtoY { token_x_amount }) => {
                msg!("Instruction: ChangeXtoY");
                Self::change_x_to_y(program_id, token_x_amount, accounts)
            }
            ContractInstruction::ChangeYtoX(ChangeYtoX { token_y_amount }) => {
                msg!("Instruction: ChangeYtoX");
                Self::change_y_to_x(program_id, token_y_amount, accounts)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{instruction::Instruction, program_stubs, rent::Rent};
    use solana_sdk::account::{create_account_for_test, create_is_signer_account_infos, Account};
    use spl_token::{
        error::TokenError,
        instruction::{
            approve, initialize_account, initialize_mint, mint_to, revoke, set_authority,
            AuthorityType,
        },
    };
    use std::sync::Arc;

    // Test program id for the swap program.
    const CONTRACT_PROGRAM_ID: Pubkey = Pubkey::new_from_array([2u8; 32]);

    struct TestSyscallStubs {}
    impl program_stubs::SyscallStubs for TestSyscallStubs {
        fn sol_invoke_signed(
            &self,
            instruction: &Instruction,
            account_infos: &[AccountInfo],
            signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            msg!("TestSyscallStubs::sol_invoke_signed()");

            let mut new_account_infos = vec![];

            // mimic check for token program in accounts
            if !account_infos.iter().any(|x| *x.key == spl_token::id()) {
                return Err(ProgramError::InvalidAccountData);
            }

            for meta in instruction.accounts.iter() {
                for account_info in account_infos.iter() {
                    if meta.pubkey == *account_info.key {
                        let mut new_account_info = account_info.clone();
                        for seeds in signers_seeds.iter() {
                            let signer =
                                Pubkey::create_program_address(seeds, &CONTRACT_PROGRAM_ID)
                                    .unwrap();
                            if *account_info.key == signer {
                                new_account_info.is_signer = true;
                            }
                        }
                        new_account_infos.push(new_account_info);
                    }
                }
            }

            spl_token::processor::Processor::process(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            )
        }
    }

    fn test_syscall_stubs() {
        use std::sync::Once;
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
        });
    }

    struct ContractAccountInfo {
        bump_seed: u8,
        authority_key: Pubkey,
        contract_key: Pubkey,
        contract_account: Account,
        pool_mint_key: Pubkey,
        pool_mint_account: Account,
        pool_wallet_x_key: Pubkey,
        pool_wallet_x_account: Account,
        user_wallet_x_key: Pubkey,
        user_wallet_x_account: Account,
        user_wallet_y_key: Pubkey,
        user_wallet_y_account: Account,
        user_wallets_authority_key: Pubkey,
        user_wallets_authority_account: Account,
    }

    impl ContractAccountInfo {
        pub fn new() -> Self {
            let contract_key = Pubkey::new_unique();
            let contract_account = Account::new(
                0,
                spl_token::state::Account::get_packed_len(),
                &CONTRACT_PROGRAM_ID,
            );
            let (authority_key, bump_seed) =
                Pubkey::find_program_address(&[&contract_key.to_bytes()[..]], &CONTRACT_PROGRAM_ID);

            let (pool_mint_key, mut pool_mint_account) = create_mint(&spl_token::id());
            /// on-chain. TODO Need execute it in processor method like "init"
            init_mint(
                &spl_token::id(),
                &pool_mint_key,
                &mut pool_mint_account,
                &authority_key,
                None,
            );

            let token_x_key = Pubkey::new_unique();
            let (authority_token_x_key, bump_seed) =
                Pubkey::find_program_address(&[], &token_x_key);

            let (token_x_mint_key, mut token_x_mint_account) = create_mint(&spl_token::id());
            init_mint(
                &spl_token::id(),
                &token_x_mint_key,
                &mut token_x_mint_account,
                &authority_token_x_key,
                None,
            );

            let user_wallets_authority_key = Pubkey::new_unique();

            let (user_wallet_x_key, user_wallet_x_account) = mint_token(
                &spl_token::id(),
                &token_x_mint_key,
                &mut token_x_mint_account,
                &authority_token_x_key,
                &user_wallets_authority_key,
                10,
            );

            let (pool_wallet_x_key, pool_wallet_x_account) = mint_token(
                &spl_token::id(),
                &token_x_mint_key,
                &mut token_x_mint_account,
                &authority_token_x_key,
                &authority_key,
                0,
            );

            let (user_wallet_y_key, user_wallet_y_account) = mint_token(
                &spl_token::id(),
                &pool_mint_key,
                &mut pool_mint_account,
                &authority_key,
                &user_wallets_authority_key,
                0,
            );

            ContractAccountInfo {
                bump_seed,
                authority_key,
                contract_key,
                contract_account,
                pool_mint_key,
                pool_mint_account,
                pool_wallet_x_key,
                pool_wallet_x_account,
                user_wallet_x_key,
                user_wallet_x_account,
                user_wallet_y_key,
                user_wallet_y_account,
                user_wallets_authority_key,
                user_wallets_authority_account: Default::default(),
            }
        }

        // pub fn initialize_swap(&mut self) -> ProgramResult {
        //     do_process_instruction(
        //         initialize(
        //             &CONTRACT_PROGRAM_ID,
        //             &spl_token::id(),
        //             &self.swap_key,
        //             &self.authority_key,
        //             &self.token_a_key,
        //             &self.token_b_key,
        //             &self.pool_mint_key,
        //             &self.pool_fee_key,
        //             &self.pool_token_key,
        //             self.fees.clone(),
        //             self.swap_curve.clone(),
        //         )
        //             .unwrap(),
        //         vec![
        //             &mut self.swap_account,
        //             &mut Account::default(),
        //             &mut self.token_a_account,
        //             &mut self.token_b_account,
        //             &mut self.pool_mint_account,
        //             &mut self.pool_fee_account,
        //             &mut self.pool_token_account,
        //             &mut Account::default(),
        //         ],
        //     )
        // }
    }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(spl_token::state::Mint::get_packed_len())
    }

    fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(spl_token::state::Account::get_packed_len())
    }

    fn do_process_instruction_with_fee_constraints(
        instruction: Instruction,
        accounts: Vec<&mut Account>,
    ) -> ProgramResult {
        test_syscall_stubs();

        // approximate the logic in the actual runtime which runs the instruction
        // and only updates accounts if the instruction is successful
        let mut account_clones = accounts.iter().map(|x| (*x).clone()).collect::<Vec<_>>();
        let mut meta = instruction
            .accounts
            .iter()
            .zip(account_clones.iter_mut())
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();
        let mut account_infos = create_is_signer_account_infos(&mut meta);
        let res = if instruction.program_id == CONTRACT_PROGRAM_ID {
            Processor::process(&instruction.program_id, &account_infos, &instruction.data)
        } else {
            spl_token::processor::Processor::process(
                &instruction.program_id,
                &account_infos,
                &instruction.data,
            )
        };

        if res.is_ok() {
            let mut account_metas = instruction
                .accounts
                .iter()
                .zip(accounts)
                .map(|(account_meta, account)| (&account_meta.pubkey, account))
                .collect::<Vec<_>>();
            for account_info in account_infos.iter_mut() {
                for account_meta in account_metas.iter_mut() {
                    if account_info.key == account_meta.0 {
                        let account = &mut account_meta.1;
                        account.owner = *account_info.owner;
                        account.lamports = **account_info.lamports.borrow();
                        account.data = account_info.data.borrow().to_vec();
                    }
                }
            }
        }
        res
    }

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut Account>,
    ) -> ProgramResult {
        do_process_instruction_with_fee_constraints(instruction, accounts)
    }

    fn mint_token(
        program_id: &Pubkey,
        mint_key: &Pubkey,
        mint_account: &mut Account,
        mint_authority_key: &Pubkey,
        account_owner_key: &Pubkey,
        amount: u64,
    ) -> (Pubkey, Account) {
        let account_key = Pubkey::new_unique();
        let mut account_account = Account::new(
            account_minimum_balance(),
            spl_token::state::Account::get_packed_len(),
            program_id,
        );
        let mut mint_authority_account = Account::default();
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_account(program_id, &account_key, mint_key, account_owner_key).unwrap(),
            vec![
                &mut account_account,
                mint_account,
                &mut mint_authority_account,
                &mut rent_sysvar_account,
            ],
        )
        .unwrap();

        if amount > 0 {
            do_process_instruction(
                mint_to(
                    program_id,
                    mint_key,
                    &account_key,
                    mint_authority_key,
                    &[],
                    amount,
                )
                .unwrap(),
                vec![
                    mint_account,
                    &mut account_account,
                    &mut mint_authority_account,
                ],
            )
            .unwrap();
        }

        (account_key, account_account)
    }

    fn create_mint(program_id: &Pubkey) -> (Pubkey, Account) {
        let mint_key = Pubkey::new_unique();
        let mut mint_account = Account::new(
            mint_minimum_balance(),
            spl_token::state::Mint::get_packed_len(),
            program_id,
        );
        (mint_key, mint_account)
    }

    fn init_mint(
        program_id: &Pubkey,
        mint_key: &Pubkey,
        mint_account: &mut Account,
        authority_key: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) {
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_mint(program_id, &mint_key, authority_key, freeze_authority, 2).unwrap(),
            vec![mint_account, &mut rent_sysvar_account],
        )
        .unwrap();
    }

    #[test]
    fn test_change_x_to_y() {
        let mut accounts = ContractAccountInfo::new();
        do_process_instruction(
            ContractInstruction::change_x_to_y(
                &CONTRACT_PROGRAM_ID,
                &spl_token::id(),
                &accounts.user_wallets_authority_key,
                &accounts.user_wallet_x_key,
                &accounts.user_wallet_y_key,
                &accounts.contract_key,
                &accounts.pool_mint_key,
                &accounts.pool_wallet_x_key,
                &accounts.authority_key,
                1,
            )
            .unwrap(),
            vec![
                &mut accounts.user_wallets_authority_account,
                &mut accounts.user_wallet_x_account,
                &mut accounts.user_wallet_y_account,
                &mut accounts.contract_account,
                &mut accounts.pool_mint_account,
                &mut accounts.pool_wallet_x_account,
            ],
        )
        .unwrap();
    }

    // #[test]
    // fn test_initialize() {
    //     let user_key = Pubkey::new_unique();
    //     let trade_fee_numerator = 1;
    //     let trade_fee_denominator = 2;
    //     let owner_trade_fee_numerator = 1;
    //     let owner_trade_fee_denominator = 10;
    //     let owner_withdraw_fee_numerator = 1;
    //     let owner_withdraw_fee_denominator = 5;
    //     let host_fee_numerator = 20;
    //     let host_fee_denominator = 100;
    //
    //     let token_a_amount = 1000;
    //     let token_b_amount = 2000;
    //     let pool_token_amount = 10;
    //
    //     let mut accounts =
    //         SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //
    //     // uninitialized token a account
    //     {
    //         let old_account = accounts.token_a_account;
    //         accounts.token_a_account = Account::new(0, 0, &spl_token::id());
    //         assert_eq!(
    //             Err(CrateError::ExpectedAccount.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_a_account = old_account;
    //     }
    //
    //     // uninitialized token b account
    //     {
    //         let old_account = accounts.token_b_account;
    //         accounts.token_b_account = Account::new(0, 0, &spl_token::id());
    //         assert_eq!(
    //             Err(CrateError::ExpectedAccount.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_b_account = old_account;
    //     }
    //
    //     // uninitialized pool mint
    //     {
    //         let old_account = accounts.pool_mint_account;
    //         accounts.pool_mint_account = Account::new(0, 0, &spl_token::id());
    //         assert_eq!(
    //             Err(CrateError::ExpectedMint.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_mint_account = old_account;
    //     }
    //
    //     // token A account owner is not swap authority
    //     {
    //         let (_token_a_key, token_a_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_a_mint_key,
    //             &mut accounts.token_a_mint_account,
    //             &user_key,
    //             &user_key,
    //             0,
    //         );
    //         let old_account = accounts.token_a_account;
    //         accounts.token_a_account = token_a_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidOwner.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_a_account = old_account;
    //     }
    //
    //     // token B account owner is not swap authority
    //     {
    //         let (_token_b_key, token_b_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_b_mint_key,
    //             &mut accounts.token_b_mint_account,
    //             &user_key,
    //             &user_key,
    //             0,
    //         );
    //         let old_account = accounts.token_b_account;
    //         accounts.token_b_account = token_b_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidOwner.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_b_account = old_account;
    //     }
    //
    //     // pool token account owner is swap authority
    //     {
    //         let (_pool_token_key, pool_token_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.pool_mint_key,
    //             &mut accounts.pool_mint_account,
    //             &accounts.authority_key,
    //             &accounts.authority_key,
    //             0,
    //         );
    //         let old_account = accounts.pool_token_account;
    //         accounts.pool_token_account = pool_token_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidOutputOwner.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_token_account = old_account;
    //     }
    //
    //     // pool fee account owner is swap authority
    //     {
    //         let (_pool_fee_key, pool_fee_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.pool_mint_key,
    //             &mut accounts.pool_mint_account,
    //             &accounts.authority_key,
    //             &accounts.authority_key,
    //             0,
    //         );
    //         let old_account = accounts.pool_fee_account;
    //         accounts.pool_fee_account = pool_fee_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidOutputOwner.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_fee_account = old_account;
    //     }
    //
    //     // pool mint authority is not swap authority
    //     {
    //         let (_pool_mint_key, pool_mint_account) =
    //             create_mint(&spl_token::id(), &user_key, None);
    //         let old_mint = accounts.pool_mint_account;
    //         accounts.pool_mint_account = pool_mint_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidOwner.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_mint_account = old_mint;
    //     }
    //
    //     // pool mint token has freeze authority
    //     {
    //         let (_pool_mint_key, pool_mint_account) =
    //             create_mint(&spl_token::id(), &accounts.authority_key, Some(&user_key));
    //         let old_mint = accounts.pool_mint_account;
    //         accounts.pool_mint_account = pool_mint_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidFreezeAuthority.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_mint_account = old_mint;
    //     }
    //
    //     // token A account owned by wrong program
    //     {
    //         let (_token_a_key, mut token_a_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_a_mint_key,
    //             &mut accounts.token_a_mint_account,
    //             &user_key,
    //             &accounts.authority_key,
    //             token_a_amount,
    //         );
    //         token_a_account.owner = CONTRACT_PROGRAM_ID;
    //         let old_account = accounts.token_a_account;
    //         accounts.token_a_account = token_a_account;
    //         assert_eq!(
    //             Err(CrateError::IncorrectTokenProgramId.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_a_account = old_account;
    //     }
    //
    //     // token B account owned by wrong program
    //     {
    //         let (_token_b_key, mut token_b_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_b_mint_key,
    //             &mut accounts.token_b_mint_account,
    //             &user_key,
    //             &accounts.authority_key,
    //             token_b_amount,
    //         );
    //         token_b_account.owner = CONTRACT_PROGRAM_ID;
    //         let old_account = accounts.token_b_account;
    //         accounts.token_b_account = token_b_account;
    //         assert_eq!(
    //             Err(CrateError::IncorrectTokenProgramId.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_b_account = old_account;
    //     }
    //
    //     // empty token A account
    //     {
    //         let (_token_a_key, token_a_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_a_mint_key,
    //             &mut accounts.token_a_mint_account,
    //             &user_key,
    //             &accounts.authority_key,
    //             0,
    //         );
    //         let old_account = accounts.token_a_account;
    //         accounts.token_a_account = token_a_account;
    //         assert_eq!(
    //             Err(CrateError::EmptySupply.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_a_account = old_account;
    //     }
    //
    //     // empty token B account
    //     {
    //         let (_token_b_key, token_b_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_b_mint_key,
    //             &mut accounts.token_b_mint_account,
    //             &user_key,
    //             &accounts.authority_key,
    //             0,
    //         );
    //         let old_account = accounts.token_b_account;
    //         accounts.token_b_account = token_b_account;
    //         assert_eq!(
    //             Err(CrateError::EmptySupply.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_b_account = old_account;
    //     }
    //
    //     // invalid pool tokens
    //     {
    //         let old_mint = accounts.pool_mint_account;
    //         let old_pool_account = accounts.pool_token_account;
    //
    //         let (_pool_mint_key, pool_mint_account) =
    //             create_mint(&spl_token::id(), &accounts.authority_key, None);
    //         accounts.pool_mint_account = pool_mint_account;
    //
    //         let (_empty_pool_token_key, empty_pool_token_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.pool_mint_key,
    //             &mut accounts.pool_mint_account,
    //             &accounts.authority_key,
    //             &user_key,
    //             0,
    //         );
    //
    //         let (_pool_token_key, pool_token_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.pool_mint_key,
    //             &mut accounts.pool_mint_account,
    //             &accounts.authority_key,
    //             &user_key,
    //             pool_token_amount,
    //         );
    //
    //         // non-empty pool token account
    //         accounts.pool_token_account = pool_token_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidSupply.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         // pool tokens already in circulation
    //         accounts.pool_token_account = empty_pool_token_account;
    //         assert_eq!(
    //             Err(CrateError::InvalidSupply.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         accounts.pool_mint_account = old_mint;
    //         accounts.pool_token_account = old_pool_account;
    //     }
    //
    //     // pool fee account has wrong mint
    //     {
    //         let (_pool_fee_key, pool_fee_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_a_mint_key,
    //             &mut accounts.token_a_mint_account,
    //             &user_key,
    //             &user_key,
    //             0,
    //         );
    //         let old_account = accounts.pool_fee_account;
    //         accounts.pool_fee_account = pool_fee_account;
    //         assert_eq!(
    //             Err(CrateError::IncorrectPoolMint.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.pool_fee_account = old_account;
    //     }
    //
    //     // token A account is delegated
    //     {
    //         do_process_instruction(
    //             approve(
    //                 &spl_token::id(),
    //                 &accounts.token_a_key,
    //                 &user_key,
    //                 &accounts.authority_key,
    //                 &[],
    //                 1,
    //             )
    //                 .unwrap(),
    //             vec![
    //                 &mut accounts.token_a_account,
    //                 &mut Account::default(),
    //                 &mut Account::default(),
    //             ],
    //         )
    //             .unwrap();
    //         assert_eq!(
    //             Err(CrateError::InvalidDelegate.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         do_process_instruction(
    //             revoke(
    //                 &spl_token::id(),
    //                 &accounts.token_a_key,
    //                 &accounts.authority_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_a_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //     }
    //
    //     // token B account is delegated
    //     {
    //         do_process_instruction(
    //             approve(
    //                 &spl_token::id(),
    //                 &accounts.token_b_key,
    //                 &user_key,
    //                 &accounts.authority_key,
    //                 &[],
    //                 1,
    //             )
    //                 .unwrap(),
    //             vec![
    //                 &mut accounts.token_b_account,
    //                 &mut Account::default(),
    //                 &mut Account::default(),
    //             ],
    //         )
    //             .unwrap();
    //         assert_eq!(
    //             Err(CrateError::InvalidDelegate.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         do_process_instruction(
    //             revoke(
    //                 &spl_token::id(),
    //                 &accounts.token_b_key,
    //                 &accounts.authority_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_b_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //     }
    //
    //     // token A account has close authority
    //     {
    //         do_process_instruction(
    //             set_authority(
    //                 &spl_token::id(),
    //                 &accounts.token_a_key,
    //                 Some(&user_key),
    //                 AuthorityType::CloseAccount,
    //                 &accounts.authority_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_a_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //         assert_eq!(
    //             Err(CrateError::InvalidCloseAuthority.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         do_process_instruction(
    //             set_authority(
    //                 &spl_token::id(),
    //                 &accounts.token_a_key,
    //                 None,
    //                 AuthorityType::CloseAccount,
    //                 &user_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_a_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //     }
    //
    //     // token B account has close authority
    //     {
    //         do_process_instruction(
    //             set_authority(
    //                 &spl_token::id(),
    //                 &accounts.token_b_key,
    //                 Some(&user_key),
    //                 AuthorityType::CloseAccount,
    //                 &accounts.authority_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_b_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //         assert_eq!(
    //             Err(CrateError::InvalidCloseAuthority.into()),
    //             accounts.initialize_swap()
    //         );
    //
    //         do_process_instruction(
    //             set_authority(
    //                 &spl_token::id(),
    //                 &accounts.token_b_key,
    //                 None,
    //                 AuthorityType::CloseAccount,
    //                 &user_key,
    //                 &[],
    //             )
    //                 .unwrap(),
    //             vec![&mut accounts.token_b_account, &mut Account::default()],
    //         )
    //             .unwrap();
    //     }
    //
    //     // wrong token program id
    //     {
    //         let wrong_program_id = Pubkey::new_unique();
    //         assert_eq!(
    //             Err(CrateError::IncorrectTokenProgramId.into()),
    //             do_process_instruction(
    //                 initialize(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &wrong_program_id,
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &accounts.pool_mint_key,
    //                     &accounts.pool_fee_key,
    //                     &accounts.pool_token_key,
    //                     accounts.fees.clone(),
    //                     accounts.swap_curve.clone(),
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut accounts.pool_mint_account,
    //                     &mut accounts.pool_fee_account,
    //                     &mut accounts.pool_token_account,
    //                     &mut Account::default(),
    //                 ],
    //             )
    //         );
    //     }
    //
    //     // create swap with same token A and B
    //     {
    //         let (_token_a_repeat_key, token_a_repeat_account) = mint_token(
    //             &spl_token::id(),
    //             &accounts.token_a_mint_key,
    //             &mut accounts.token_a_mint_account,
    //             &user_key,
    //             &accounts.authority_key,
    //             10,
    //         );
    //         let old_account = accounts.token_b_account;
    //         accounts.token_b_account = token_a_repeat_account;
    //         assert_eq!(
    //             Err(CrateError::RepeatedMint.into()),
    //             accounts.initialize_swap()
    //         );
    //         accounts.token_b_account = old_account;
    //     }
    //
    //     // create valid swap
    //     accounts.initialize_swap().unwrap();
    //
    //     // create invalid flat swap
    //     {
    //         let token_b_price = 0;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::ConstantPrice,
    //             calculator: Arc::new(ConstantPriceCurve { token_b_price }),
    //         };
    //         let mut accounts =
    //             SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //         assert_eq!(
    //             Err(CrateError::InvalidCurve.into()),
    //             accounts.initialize_swap()
    //         );
    //     }
    //
    //     // create valid flat swap
    //     {
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let token_b_price = 10_000;
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::ConstantPrice,
    //             calculator: Arc::new(ConstantPriceCurve { token_b_price }),
    //         };
    //         let mut accounts =
    //             SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //         accounts.initialize_swap().unwrap();
    //     }
    //
    //     // create invalid offset swap
    //     {
    //         let token_b_offset = 0;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::Offset,
    //             calculator: Arc::new(OffsetCurve { token_b_offset }),
    //         };
    //         let mut accounts =
    //             SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //         assert_eq!(
    //             Err(CrateError::InvalidCurve.into()),
    //             accounts.initialize_swap()
    //         );
    //     }
    //
    //     // create valid offset swap
    //     {
    //         let token_b_offset = 10;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::Offset,
    //             calculator: Arc::new(OffsetCurve { token_b_offset }),
    //         };
    //         let mut accounts =
    //             SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //         accounts.initialize_swap().unwrap();
    //     }
    //
    //     // wrong owner key in constraint
    //     {
    //         let new_key = Pubkey::new_unique();
    //         let trade_fee_numerator = 25;
    //         let trade_fee_denominator = 10000;
    //         let owner_trade_fee_numerator = 5;
    //         let owner_trade_fee_denominator = 10000;
    //         let host_fee_numerator = 20;
    //         let host_fee_denominator = 100;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let curve = ConstantProductCurve {};
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::ConstantProduct,
    //             calculator: Arc::new(curve),
    //         };
    //         let owner_key = &new_key.to_string();
    //         let valid_curve_types = &[CurveType::ConstantProduct];
    //         let constraints = Some(SwapConstraints {
    //             owner_key,
    //             valid_curve_types,
    //             fees: &fees,
    //         });
    //         let mut accounts = SwapAccountInfo::new(
    //             &user_key,
    //             fees.clone(),
    //             swap_curve,
    //             token_a_amount,
    //             token_b_amount,
    //         );
    //         assert_eq!(
    //             Err(CrateError::InvalidOwner.into()),
    //             do_process_instruction_with_fee_constraints(
    //                 initialize(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &spl_token::id(),
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &accounts.pool_mint_key,
    //                     &accounts.pool_fee_key,
    //                     &accounts.pool_token_key,
    //                     accounts.fees.clone(),
    //                     accounts.swap_curve.clone(),
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut accounts.pool_mint_account,
    //                     &mut accounts.pool_fee_account,
    //                     &mut accounts.pool_token_account,
    //                     &mut Account::default(),
    //                 ],
    //                 &constraints,
    //             )
    //         );
    //     }
    //
    //     // wrong fee in constraint
    //     {
    //         let trade_fee_numerator = 25;
    //         let trade_fee_denominator = 10000;
    //         let owner_trade_fee_numerator = 5;
    //         let owner_trade_fee_denominator = 10000;
    //         let host_fee_numerator = 20;
    //         let host_fee_denominator = 100;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let curve = ConstantProductCurve {};
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::ConstantProduct,
    //             calculator: Arc::new(curve),
    //         };
    //         let owner_key = &user_key.to_string();
    //         let valid_curve_types = &[CurveType::ConstantProduct];
    //         let constraints = Some(SwapConstraints {
    //             owner_key,
    //             valid_curve_types,
    //             fees: &fees,
    //         });
    //         let mut bad_fees = fees.clone();
    //         bad_fees.trade_fee_numerator = trade_fee_numerator - 1;
    //         let mut accounts = SwapAccountInfo::new(
    //             &user_key,
    //             bad_fees,
    //             swap_curve,
    //             token_a_amount,
    //             token_b_amount,
    //         );
    //         assert_eq!(
    //             Err(CrateError::InvalidFee.into()),
    //             do_process_instruction_with_fee_constraints(
    //                 initialize(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &spl_token::id(),
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &accounts.pool_mint_key,
    //                     &accounts.pool_fee_key,
    //                     &accounts.pool_token_key,
    //                     accounts.fees.clone(),
    //                     accounts.swap_curve.clone(),
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut accounts.pool_mint_account,
    //                     &mut accounts.pool_fee_account,
    //                     &mut accounts.pool_token_account,
    //                     &mut Account::default(),
    //                 ],
    //                 &constraints,
    //             )
    //         );
    //     }
    //
    //     // create valid swap with constraints
    //     {
    //         let trade_fee_numerator = 25;
    //         let trade_fee_denominator = 10000;
    //         let owner_trade_fee_numerator = 5;
    //         let owner_trade_fee_denominator = 10000;
    //         let host_fee_numerator = 20;
    //         let host_fee_denominator = 100;
    //         let fees = Fees {
    //             trade_fee_numerator,
    //             trade_fee_denominator,
    //             owner_trade_fee_numerator,
    //             owner_trade_fee_denominator,
    //             owner_withdraw_fee_numerator,
    //             owner_withdraw_fee_denominator,
    //             host_fee_numerator,
    //             host_fee_denominator,
    //         };
    //         let curve = ConstantProductCurve {};
    //         let swap_curve = SwapCurve {
    //             curve_type: CurveType::ConstantProduct,
    //             calculator: Arc::new(curve),
    //         };
    //         let owner_key = &user_key.to_string();
    //         let valid_curve_types = &[CurveType::ConstantProduct];
    //         let constraints = Some(SwapConstraints {
    //             owner_key,
    //             valid_curve_types,
    //             fees: &fees,
    //         });
    //         let mut accounts = SwapAccountInfo::new(
    //             &user_key,
    //             fees.clone(),
    //             swap_curve,
    //             token_a_amount,
    //             token_b_amount,
    //         );
    //         do_process_instruction_with_fee_constraints(
    //             initialize(
    //                 &CONTRACT_PROGRAM_ID,
    //                 &spl_token::id(),
    //                 &accounts.swap_key,
    //                 &accounts.authority_key,
    //                 &accounts.token_a_key,
    //                 &accounts.token_b_key,
    //                 &accounts.pool_mint_key,
    //                 &accounts.pool_fee_key,
    //                 &accounts.pool_token_key,
    //                 accounts.fees,
    //                 accounts.swap_curve.clone(),
    //             )
    //                 .unwrap(),
    //             vec![
    //                 &mut accounts.swap_account,
    //                 &mut Account::default(),
    //                 &mut accounts.token_a_account,
    //                 &mut accounts.token_b_account,
    //                 &mut accounts.pool_mint_account,
    //                 &mut accounts.pool_fee_account,
    //                 &mut accounts.pool_token_account,
    //                 &mut Account::default(),
    //             ],
    //             &constraints,
    //         )
    //             .unwrap();
    //     }
    //
    //     // create again
    //     {
    //         assert_eq!(
    //             Err(CrateError::AlreadyInUse.into()),
    //             accounts.initialize_swap()
    //         );
    //     }
    //     let swap_state = SwapVersion::unpack(&accounts.swap_account.data).unwrap();
    //     assert!(swap_state.is_initialized());
    //     assert_eq!(swap_state.bump_seed(), accounts.bump_seed);
    //     assert_eq!(
    //         swap_state.swap_curve().curve_type,
    //         accounts.swap_curve.curve_type
    //     );
    //     assert_eq!(*swap_state.token_a_account(), accounts.token_a_key);
    //     assert_eq!(*swap_state.token_b_account(), accounts.token_b_key);
    //     assert_eq!(*swap_state.pool_mint(), accounts.pool_mint_key);
    //     assert_eq!(*swap_state.token_a_mint(), accounts.token_a_mint_key);
    //     assert_eq!(*swap_state.token_b_mint(), accounts.token_b_mint_key);
    //     assert_eq!(*swap_state.pool_fee_account(), accounts.pool_fee_key);
    //     let token_a = spl_token::state::Account::unpack(&accounts.token_a_account.data).unwrap();
    //     assert_eq!(token_a.amount, token_a_amount);
    //     let token_b = spl_token::state::Account::unpack(&accounts.token_b_account.data).unwrap();
    //     assert_eq!(token_b.amount, token_b_amount);
    //     let pool_account =
    //         spl_token::state::Account::unpack(&accounts.pool_token_account.data).unwrap();
    //     let pool_mint = spl_token::state::Mint::unpack(&accounts.pool_mint_account.data).unwrap();
    //     assert_eq!(pool_mint.supply, pool_account.amount);
    // }

    // #[test]
    // fn test_deposit() {
    //     let user_key = Pubkey::new_unique();
    //     let depositor_key = Pubkey::new_unique();
    //     let trade_fee_numerator = 1;
    //     let trade_fee_denominator = 2;
    //     let owner_trade_fee_numerator = 1;
    //     let owner_trade_fee_denominator = 10;
    //     let owner_withdraw_fee_numerator = 1;
    //     let owner_withdraw_fee_denominator = 5;
    //     let host_fee_numerator = 20;
    //     let host_fee_denominator = 100;
    //
    //     let fees = Fees {
    //         trade_fee_numerator,
    //         trade_fee_denominator,
    //         owner_trade_fee_numerator,
    //         owner_trade_fee_denominator,
    //         owner_withdraw_fee_numerator,
    //         owner_withdraw_fee_denominator,
    //         host_fee_numerator,
    //         host_fee_denominator,
    //     };
    //
    //     let token_a_amount = 1000;
    //     let token_b_amount = 9000;
    //     let curve_type = CurveType::ConstantProduct;
    //     let swap_curve = SwapCurve {
    //         curve_type,
    //         calculator: Arc::new(ConstantProductCurve {}),
    //     };
    //
    //     let mut accounts =
    //         SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //
    //     // depositing 10% of the current pool amount in token A and B means
    //     // that our pool tokens will be worth 1 / 10 of the current pool amount
    //     let pool_amount = INITIAL_SWAP_POOL_AMOUNT / 10;
    //     let deposit_a = token_a_amount / 10;
    //     let deposit_b = token_b_amount / 10;
    //
    //     // swap not initialized
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         assert_eq!(
    //             Err(ProgramError::UninitializedAccount),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     accounts.initialize_swap().unwrap();
    //
    //     // wrong owner for swap account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let old_swap_account = accounts.swap_account;
    //         let mut wrong_swap_account = old_swap_account.clone();
    //         wrong_swap_account.owner = spl_token::id();
    //         accounts.swap_account = wrong_swap_account;
    //         assert_eq!(
    //             Err(ProgramError::IncorrectProgramId),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //         accounts.swap_account = old_swap_account;
    //     }
    //
    //     // wrong bump seed for authority_key
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let old_authority = accounts.authority_key;
    //         let (bad_authority_key, _bump_seed) = Pubkey::find_program_address(
    //             &[&accounts.swap_key.to_bytes()[..]],
    //             &spl_token::id(),
    //         );
    //         accounts.authority_key = bad_authority_key;
    //         assert_eq!(
    //             Err(CrateError::InvalidProgramAddress.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //         accounts.authority_key = old_authority;
    //     }
    //
    //     // not enough token A
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &depositor_key,
    //             deposit_a / 2,
    //             deposit_b,
    //             0,
    //         );
    //         assert_eq!(
    //             Err(TokenError::InsufficientFunds.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // not enough token B
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &depositor_key,
    //             deposit_a,
    //             deposit_b / 2,
    //             0,
    //         );
    //         assert_eq!(
    //             Err(TokenError::InsufficientFunds.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // wrong swap token accounts
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         assert_eq!(
    //             Err(TokenError::MintMismatch.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // wrong pool token account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             _pool_key,
    //             mut _pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let (
    //             wrong_token_key,
    //             mut wrong_token_account,
    //             _token_b_key,
    //             mut _token_b_account,
    //             _pool_key,
    //             mut _pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         assert_eq!(
    //             Err(TokenError::MintMismatch.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &wrong_token_key,
    //                 &mut wrong_token_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // no approval
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let user_transfer_authority_key = Pubkey::new_unique();
    //         assert_eq!(
    //             Err(TokenError::OwnerMismatch.into()),
    //             do_process_instruction(
    //                 deposit_all_token_types(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &spl_token::id(),
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &user_transfer_authority_key,
    //                     &token_a_key,
    //                     &token_b_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &accounts.pool_mint_key,
    //                     &pool_key,
    //                     DepositAllTokenTypes {
    //                         pool_token_amount: pool_amount.try_into().unwrap(),
    //                         maximum_token_a_amount: deposit_a,
    //                         maximum_token_b_amount: deposit_b,
    //                     },
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut Account::default(),
    //                     &mut token_a_account,
    //                     &mut token_b_account,
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut accounts.pool_mint_account,
    //                     &mut pool_account,
    //                     &mut Account::default(),
    //                 ],
    //             )
    //         );
    //     }
    //
    //     // wrong token program id
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let wrong_key = Pubkey::new_unique();
    //         assert_eq!(
    //             Err(CrateError::IncorrectTokenProgramId.into()),
    //             do_process_instruction(
    //                 deposit_all_token_types(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &wrong_key,
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &accounts.authority_key,
    //                     &token_a_key,
    //                     &token_b_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &accounts.pool_mint_key,
    //                     &pool_key,
    //                     DepositAllTokenTypes {
    //                         pool_token_amount: pool_amount.try_into().unwrap(),
    //                         maximum_token_a_amount: deposit_a,
    //                         maximum_token_b_amount: deposit_b,
    //                     },
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut Account::default(),
    //                     &mut token_a_account,
    //                     &mut token_b_account,
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut accounts.pool_mint_account,
    //                     &mut pool_account,
    //                     &mut Account::default(),
    //                 ],
    //             )
    //         );
    //     }
    //
    //     // wrong swap token accounts
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //
    //         let old_a_key = accounts.token_a_key;
    //         let old_a_account = accounts.token_a_account;
    //
    //         accounts.token_a_key = token_a_key;
    //         accounts.token_a_account = token_a_account.clone();
    //
    //         // wrong swap token a account
    //         assert_eq!(
    //             Err(CrateError::IncorrectSwapAccount.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //
    //         accounts.token_a_key = old_a_key;
    //         accounts.token_a_account = old_a_account;
    //
    //         let old_b_key = accounts.token_b_key;
    //         let old_b_account = accounts.token_b_account;
    //
    //         accounts.token_b_key = token_b_key;
    //         accounts.token_b_account = token_b_account.clone();
    //
    //         // wrong swap token b account
    //         assert_eq!(
    //             Err(CrateError::IncorrectSwapAccount.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //
    //         accounts.token_b_key = old_b_key;
    //         accounts.token_b_account = old_b_account;
    //     }
    //
    //     // wrong mint
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let (pool_mint_key, pool_mint_account) =
    //             create_mint(&spl_token::id(), &accounts.authority_key, None);
    //         let old_pool_key = accounts.pool_mint_key;
    //         let old_pool_account = accounts.pool_mint_account;
    //         accounts.pool_mint_key = pool_mint_key;
    //         accounts.pool_mint_account = pool_mint_account;
    //
    //         assert_eq!(
    //             Err(CrateError::IncorrectPoolMint.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //
    //         accounts.pool_mint_key = old_pool_key;
    //         accounts.pool_mint_account = old_pool_account;
    //     }
    //
    //     // deposit 1 pool token fails beacuse it equates to 0 swap tokens
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         assert_eq!(
    //             Err(CrateError::ZeroTradingTokens.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 1,
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // slippage exceeded
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         // maximum A amount in too low
    //         assert_eq!(
    //             Err(CrateError::ExceededSlippage.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a / 10,
    //                 deposit_b,
    //             )
    //         );
    //         // maximum B amount in too low
    //         assert_eq!(
    //             Err(CrateError::ExceededSlippage.into()),
    //             accounts.deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b / 10,
    //             )
    //         );
    //     }
    //
    //     // invalid input: can't use swap pool tokens as source
    //     {
    //         let (
    //             _token_a_key,
    //             _token_a_account,
    //             _token_b_key,
    //             _token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         let swap_token_a_key = accounts.token_a_key;
    //         let mut swap_token_a_account = accounts.get_token_account(&swap_token_a_key).clone();
    //         let swap_token_b_key = accounts.token_b_key;
    //         let mut swap_token_b_account = accounts.get_token_account(&swap_token_b_key).clone();
    //         let authority_key = accounts.authority_key;
    //         assert_eq!(
    //             Err(CrateError::InvalidInput.into()),
    //             accounts.deposit_all_token_types(
    //                 &authority_key,
    //                 &swap_token_a_key,
    //                 &mut swap_token_a_account,
    //                 &swap_token_b_key,
    //                 &mut swap_token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //         );
    //     }
    //
    //     // correctly deposit
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &depositor_key, deposit_a, deposit_b, 0);
    //         accounts
    //             .deposit_all_token_types(
    //                 &depositor_key,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 pool_amount.try_into().unwrap(),
    //                 deposit_a,
    //                 deposit_b,
    //             )
    //             .unwrap();
    //
    //         let swap_token_a =
    //             spl_token::state::Account::unpack(&accounts.token_a_account.data).unwrap();
    //         assert_eq!(swap_token_a.amount, deposit_a + token_a_amount);
    //         let swap_token_b =
    //             spl_token::state::Account::unpack(&accounts.token_b_account.data).unwrap();
    //         assert_eq!(swap_token_b.amount, deposit_b + token_b_amount);
    //         let token_a = spl_token::state::Account::unpack(&token_a_account.data).unwrap();
    //         assert_eq!(token_a.amount, 0);
    //         let token_b = spl_token::state::Account::unpack(&token_b_account.data).unwrap();
    //         assert_eq!(token_b.amount, 0);
    //         let pool_account = spl_token::state::Account::unpack(&pool_account.data).unwrap();
    //         let swap_pool_account =
    //             spl_token::state::Account::unpack(&accounts.pool_token_account.data).unwrap();
    //         let pool_mint =
    //             spl_token::state::Mint::unpack(&accounts.pool_mint_account.data).unwrap();
    //         assert_eq!(
    //             pool_mint.supply,
    //             pool_account.amount + swap_pool_account.amount
    //         );
    //     }
    // }

    // #[test]
    // fn test_withdraw() {
    //     let user_key = Pubkey::new_unique();
    //     let trade_fee_numerator = 1;
    //     let trade_fee_denominator = 2;
    //     let owner_trade_fee_numerator = 1;
    //     let owner_trade_fee_denominator = 10;
    //     let owner_withdraw_fee_numerator = 1;
    //     let owner_withdraw_fee_denominator = 5;
    //     let host_fee_numerator = 7;
    //     let host_fee_denominator = 100;
    //
    //     let fees = Fees {
    //         trade_fee_numerator,
    //         trade_fee_denominator,
    //         owner_trade_fee_numerator,
    //         owner_trade_fee_denominator,
    //         owner_withdraw_fee_numerator,
    //         owner_withdraw_fee_denominator,
    //         host_fee_numerator,
    //         host_fee_denominator,
    //     };
    //
    //     let token_a_amount = 1000;
    //     let token_b_amount = 2000;
    //     let curve_type = CurveType::ConstantProduct;
    //     let swap_curve = SwapCurve {
    //         curve_type,
    //         calculator: Arc::new(ConstantProductCurve {}),
    //     };
    //
    //     let withdrawer_key = Pubkey::new_unique();
    //     let initial_a = token_a_amount / 10;
    //     let initial_b = token_b_amount / 10;
    //     let initial_pool = swap_curve.calculator.new_pool_supply() / 10;
    //     let withdraw_amount = initial_pool / 4;
    //     let minimum_token_a_amount = initial_a / 40;
    //     let minimum_token_b_amount = initial_b / 40;
    //
    //     let mut accounts =
    //         SwapAccountInfo::new(&user_key, fees, swap_curve, token_a_amount, token_b_amount);
    //
    //     // swap not initialized
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &withdrawer_key, initial_a, initial_b, 0);
    //         assert_eq!(
    //             Err(ProgramError::UninitializedAccount),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //     }
    //
    //     accounts.initialize_swap().unwrap();
    //
    //     // wrong owner for swap account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &withdrawer_key, initial_a, initial_b, 0);
    //         let old_swap_account = accounts.swap_account;
    //         let mut wrong_swap_account = old_swap_account.clone();
    //         wrong_swap_account.owner = spl_token::id();
    //         accounts.swap_account = wrong_swap_account;
    //         assert_eq!(
    //             Err(ProgramError::IncorrectProgramId),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //         accounts.swap_account = old_swap_account;
    //     }
    //
    //     // wrong bump seed for authority_key
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &withdrawer_key, initial_a, initial_b, 0);
    //         let old_authority = accounts.authority_key;
    //         let (bad_authority_key, _bump_seed) = Pubkey::find_program_address(
    //             &[&accounts.swap_key.to_bytes()[..]],
    //             &spl_token::id(),
    //         );
    //         accounts.authority_key = bad_authority_key;
    //         assert_eq!(
    //             Err(CrateError::InvalidProgramAddress.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //         accounts.authority_key = old_authority;
    //     }
    //
    //     // not enough pool tokens
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             to_u64(withdraw_amount).unwrap() / 2u64,
    //         );
    //         assert_eq!(
    //             Err(TokenError::InsufficientFunds.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount / 2,
    //                 minimum_token_b_amount / 2,
    //             )
    //         );
    //     }
    //
    //     // wrong token a / b accounts
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         assert_eq!(
    //             Err(TokenError::MintMismatch.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //     }
    //
    //     // wrong pool token account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             _pool_key,
    //             _pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         let (
    //             wrong_token_a_key,
    //             mut wrong_token_a_account,
    //             _token_b_key,
    //             _token_b_account,
    //             _pool_key,
    //             _pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             withdraw_amount.try_into().unwrap(),
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         assert_eq!(
    //             Err(TokenError::MintMismatch.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &wrong_token_a_key,
    //                 &mut wrong_token_a_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //     }
    //
    //     // wrong pool fee account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             wrong_pool_key,
    //             wrong_pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         let (
    //             _token_a_key,
    //             _token_a_account,
    //             _token_b_key,
    //             _token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         let old_pool_fee_account = accounts.pool_fee_account;
    //         let old_pool_fee_key = accounts.pool_fee_key;
    //         accounts.pool_fee_account = wrong_pool_account;
    //         accounts.pool_fee_key = wrong_pool_key;
    //         assert_eq!(
    //             Err(CrateError::IncorrectFeeAccount.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             ),
    //         );
    //         accounts.pool_fee_account = old_pool_fee_account;
    //         accounts.pool_fee_key = old_pool_fee_key;
    //     }
    //
    //     // no approval
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             0,
    //             0,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         let user_transfer_authority_key = Pubkey::new_unique();
    //         assert_eq!(
    //             Err(TokenError::OwnerMismatch.into()),
    //             do_process_instruction(
    //                 withdraw_all_token_types(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &spl_token::id(),
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &user_transfer_authority_key,
    //                     &accounts.pool_mint_key,
    //                     &accounts.pool_fee_key,
    //                     &pool_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &token_a_key,
    //                     &token_b_key,
    //                     WithdrawAllTokenTypes {
    //                         pool_token_amount: withdraw_amount.try_into().unwrap(),
    //                         minimum_token_a_amount,
    //                         minimum_token_b_amount,
    //                     }
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut Account::default(),
    //                     &mut accounts.pool_mint_account,
    //                     &mut pool_account,
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut token_a_account,
    //                     &mut token_b_account,
    //                     &mut accounts.pool_fee_account,
    //                     &mut Account::default(),
    //                 ],
    //             )
    //         );
    //     }
    //
    //     // wrong token program id
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             withdraw_amount.try_into().unwrap(),
    //         );
    //         let wrong_key = Pubkey::new_unique();
    //         assert_eq!(
    //             Err(CrateError::IncorrectTokenProgramId.into()),
    //             do_process_instruction(
    //                 withdraw_all_token_types(
    //                     &CONTRACT_PROGRAM_ID,
    //                     &wrong_key,
    //                     &accounts.swap_key,
    //                     &accounts.authority_key,
    //                     &accounts.authority_key,
    //                     &accounts.pool_mint_key,
    //                     &accounts.pool_fee_key,
    //                     &pool_key,
    //                     &accounts.token_a_key,
    //                     &accounts.token_b_key,
    //                     &token_a_key,
    //                     &token_b_key,
    //                     WithdrawAllTokenTypes {
    //                         pool_token_amount: withdraw_amount.try_into().unwrap(),
    //                         minimum_token_a_amount,
    //                         minimum_token_b_amount,
    //                     },
    //                 )
    //                     .unwrap(),
    //                 vec![
    //                     &mut accounts.swap_account,
    //                     &mut Account::default(),
    //                     &mut Account::default(),
    //                     &mut accounts.pool_mint_account,
    //                     &mut pool_account,
    //                     &mut accounts.token_a_account,
    //                     &mut accounts.token_b_account,
    //                     &mut token_a_account,
    //                     &mut token_b_account,
    //                     &mut accounts.pool_fee_account,
    //                     &mut Account::default(),
    //                 ],
    //             )
    //         );
    //     }
    //
    //     // wrong swap token accounts
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //
    //         let old_a_key = accounts.token_a_key;
    //         let old_a_account = accounts.token_a_account;
    //
    //         accounts.token_a_key = token_a_key;
    //         accounts.token_a_account = token_a_account.clone();
    //
    //         // wrong swap token a account
    //         assert_eq!(
    //             Err(CrateError::IncorrectSwapAccount.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //
    //         accounts.token_a_key = old_a_key;
    //         accounts.token_a_account = old_a_account;
    //
    //         let old_b_key = accounts.token_b_key;
    //         let old_b_account = accounts.token_b_account;
    //
    //         accounts.token_b_key = token_b_key;
    //         accounts.token_b_account = token_b_account.clone();
    //
    //         // wrong swap token b account
    //         assert_eq!(
    //             Err(CrateError::IncorrectSwapAccount.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //
    //         accounts.token_b_key = old_b_key;
    //         accounts.token_b_account = old_b_account;
    //     }
    //
    //     // wrong mint
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //         let (pool_mint_key, pool_mint_account) =
    //             create_mint(&spl_token::id(), &accounts.authority_key, None);
    //         let old_pool_key = accounts.pool_mint_key;
    //         let old_pool_account = accounts.pool_mint_account;
    //         accounts.pool_mint_key = pool_mint_key;
    //         accounts.pool_mint_account = pool_mint_account;
    //
    //         assert_eq!(
    //             Err(CrateError::IncorrectPoolMint.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //
    //         accounts.pool_mint_key = old_pool_key;
    //         accounts.pool_mint_account = old_pool_account;
    //     }
    //
    //     // withdrawing 1 pool token fails because it equates to 0 output tokens
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //         assert_eq!(
    //             Err(CrateError::ZeroTradingTokens.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 1,
    //                 0,
    //                 0,
    //             )
    //         );
    //     }
    //
    //     // slippage exceeded
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //         // minimum A amount out too high
    //         assert_eq!(
    //             Err(CrateError::ExceededSlippage.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount * 10,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //         // minimum B amount out too high
    //         assert_eq!(
    //             Err(CrateError::ExceededSlippage.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount * 10,
    //             )
    //         );
    //     }
    //
    //     // invalid input: can't use swap pool tokens as destination
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //         let swap_token_a_key = accounts.token_a_key;
    //         let mut swap_token_a_account = accounts.get_token_account(&swap_token_a_key).clone();
    //         assert_eq!(
    //             Err(CrateError::InvalidInput.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &swap_token_a_key,
    //                 &mut swap_token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //         let swap_token_b_key = accounts.token_b_key;
    //         let mut swap_token_b_account = accounts.get_token_account(&swap_token_b_key).clone();
    //         assert_eq!(
    //             Err(CrateError::InvalidInput.into()),
    //             accounts.withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &swap_token_b_key,
    //                 &mut swap_token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //         );
    //     }
    //
    //     // correct withdrawal
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             pool_key,
    //             mut pool_account,
    //         ) = accounts.setup_token_accounts(
    //             &user_key,
    //             &withdrawer_key,
    //             initial_a,
    //             initial_b,
    //             initial_pool.try_into().unwrap(),
    //         );
    //
    //         accounts
    //             .withdraw_all_token_types(
    //                 &withdrawer_key,
    //                 &pool_key,
    //                 &mut pool_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 withdraw_amount.try_into().unwrap(),
    //                 minimum_token_a_amount,
    //                 minimum_token_b_amount,
    //             )
    //             .unwrap();
    //
    //         let swap_token_a =
    //             spl_token::state::Account::unpack(&accounts.token_a_account.data).unwrap();
    //         let swap_token_b =
    //             spl_token::state::Account::unpack(&accounts.token_b_account.data).unwrap();
    //         let pool_mint =
    //             spl_token::state::Mint::unpack(&accounts.pool_mint_account.data).unwrap();
    //         let withdraw_fee = accounts.fees.owner_withdraw_fee(withdraw_amount).unwrap();
    //         let results = accounts
    //             .swap_curve
    //             .calculator
    //             .pool_tokens_to_trading_tokens(
    //                 withdraw_amount - withdraw_fee,
    //                 pool_mint.supply.try_into().unwrap(),
    //                 swap_token_a.amount.try_into().unwrap(),
    //                 swap_token_b.amount.try_into().unwrap(),
    //                 RoundDirection::Floor,
    //             )
    //             .unwrap();
    //         assert_eq!(
    //             swap_token_a.amount,
    //             token_a_amount - to_u64(results.token_a_amount).unwrap()
    //         );
    //         assert_eq!(
    //             swap_token_b.amount,
    //             token_b_amount - to_u64(results.token_b_amount).unwrap()
    //         );
    //         let token_a = spl_token::state::Account::unpack(&token_a_account.data).unwrap();
    //         assert_eq!(
    //             token_a.amount,
    //             initial_a + to_u64(results.token_a_amount).unwrap()
    //         );
    //         let token_b = spl_token::state::Account::unpack(&token_b_account.data).unwrap();
    //         assert_eq!(
    //             token_b.amount,
    //             initial_b + to_u64(results.token_b_amount).unwrap()
    //         );
    //         let pool_account = spl_token::state::Account::unpack(&pool_account.data).unwrap();
    //         assert_eq!(
    //             pool_account.amount,
    //             to_u64(initial_pool - withdraw_amount).unwrap()
    //         );
    //         let fee_account =
    //             spl_token::state::Account::unpack(&accounts.pool_fee_account.data).unwrap();
    //         assert_eq!(
    //             fee_account.amount,
    //             TryInto::<u64>::try_into(withdraw_fee).unwrap()
    //         );
    //     }
    //
    //     // correct withdrawal from fee account
    //     {
    //         let (
    //             token_a_key,
    //             mut token_a_account,
    //             token_b_key,
    //             mut token_b_account,
    //             _pool_key,
    //             mut _pool_account,
    //         ) = accounts.setup_token_accounts(&user_key, &withdrawer_key, 0, 0, 0);
    //
    //         let pool_fee_key = accounts.pool_fee_key;
    //         let mut pool_fee_account = accounts.pool_fee_account.clone();
    //         let fee_account = spl_token::state::Account::unpack(&pool_fee_account.data).unwrap();
    //         let pool_fee_amount = fee_account.amount;
    //
    //         accounts
    //             .withdraw_all_token_types(
    //                 &user_key,
    //                 &pool_fee_key,
    //                 &mut pool_fee_account,
    //                 &token_a_key,
    //                 &mut token_a_account,
    //                 &token_b_key,
    //                 &mut token_b_account,
    //                 pool_fee_amount,
    //                 0,
    //                 0,
    //             )
    //             .unwrap();
    //
    //         let swap_token_a =
    //             spl_token::state::Account::unpack(&accounts.token_a_account.data).unwrap();
    //         let swap_token_b =
    //             spl_token::state::Account::unpack(&accounts.token_b_account.data).unwrap();
    //         let pool_mint =
    //             spl_token::state::Mint::unpack(&accounts.pool_mint_account.data).unwrap();
    //         let results = accounts
    //             .swap_curve
    //             .calculator
    //             .pool_tokens_to_trading_tokens(
    //                 pool_fee_amount.try_into().unwrap(),
    //                 pool_mint.supply.try_into().unwrap(),
    //                 swap_token_a.amount.try_into().unwrap(),
    //                 swap_token_b.amount.try_into().unwrap(),
    //                 RoundDirection::Floor,
    //             )
    //             .unwrap();
    //         let token_a = spl_token::state::Account::unpack(&token_a_account.data).unwrap();
    //         assert_eq!(
    //             token_a.amount,
    //             TryInto::<u64>::try_into(results.token_a_amount).unwrap()
    //         );
    //         let token_b = spl_token::state::Account::unpack(&token_b_account.data).unwrap();
    //         assert_eq!(
    //             token_b.amount,
    //             TryInto::<u64>::try_into(results.token_b_amount).unwrap()
    //         );
    //     }
    // }
}
