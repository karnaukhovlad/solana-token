use crate::error::CrateError;
use crate::instruction::ContractInstruction;
use borsh::BorshDeserialize;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

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
        pool_contract: AccountInfo<'a>,
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let pool_contract_bytes = pool_contract.key.to_bytes();
        let authority_signature_seeds = [&pool_contract_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = spl_token::instruction::burn(
            &spl_token::id(),
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[burn_account, mint, authority], signers)
    }

    /// Issue a spl_token `MintTo` instruction.
    pub fn token_mint_to<'a>(
        pool_contract: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let pool_contract_bytes = pool_contract.key.to_bytes();
        let authority_signature_seeds = [&pool_contract_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[mint, destination, authority], signers)
    }

    /// Issue a spl_token `Transfer` instruction.
    pub fn token_transfer<'a>(
        pool_contract: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        amount: u64,
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let pool_contract_bytes = pool_contract.key.to_bytes();
        let authority_signature_seeds = [&pool_contract_bytes[..32], &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];
        let ix = spl_token::instruction::transfer(
            &spl_token::id(),
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;
        invoke(&ix, &[source, destination, authority])
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
        let user_wallet_y_info = next_account_info(account_info_iter)?;
        let pool_contract_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let pool_wallet_x_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !user_wallets_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if dbg!(program_id) != dbg!(pool_contract_info.owner) {
            msg!("Pool contract provided is not owned by the program");
            return Err(ProgramError::IncorrectProgramId);
        }

        let (program_authority_id, bump_seed) =
            Pubkey::find_program_address(&[&pool_contract_info.key.to_bytes()], program_id);
        if *authority_info.key != program_authority_id {
            return Err(CrateError::InvalidProgramAddress.into());
        }

        msg!("Transferring");
        Self::token_transfer(
            pool_contract_info.clone(),
            user_wallet_x_info.clone(),
            pool_wallet_x_info.clone(),
            user_wallets_authority_info.clone(),
            token_x_amount,
            bump_seed,
        )?;
        msg!("Minting");
        Self::token_mint_to(
            pool_contract_info.clone(),
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
        let account_info_iter = &mut accounts.iter();

        let user_wallets_authority_info = next_account_info(account_info_iter)?;
        let user_wallet_x_info = next_account_info(account_info_iter)?;
        let user_wallet_y_info = next_account_info(account_info_iter)?;
        let pool_contract_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let pool_wallet_x_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !user_wallets_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if program_id != pool_contract_info.owner {
            msg!("Pool contract provided is not owned by the program");
            return Err(ProgramError::IncorrectProgramId);
        }

        let (program_authority_id, bump_seed) =
            Pubkey::find_program_address(&[&pool_contract_info.key.to_bytes()], program_id);
        if *authority_info.key != program_authority_id {
            return Err(CrateError::InvalidProgramAddress.into());
        }

        Self::token_transfer(
            pool_contract_info.clone(),
            pool_wallet_x_info.clone(),
            user_wallet_x_info.clone(),
            authority_info.clone(),
            token_y_amount,
            bump_seed,
        )?;
        Self::token_burn(
            pool_contract_info.clone(),
            user_wallet_y_info.clone(),
            pool_mint_info.clone(),
            user_wallets_authority_info.clone(),
            token_y_amount,
            bump_seed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction as CrateInstruction;
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
            Processor::process_instruction(
                &instruction.program_id,
                &account_infos,
                &instruction.data,
            )
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

    // #[test]
    // fn test_contract() {
    //     let mut accounts = ContractAccountInfo::new();
    //     do_process_instruction(
    //         CrateInstruction::change_x_to_y(
    //             &CONTRACT_PROGRAM_ID,
    //             &accounts.user_wallets_authority_key,
    //             &accounts.user_wallet_x_key,
    //             &accounts.user_wallet_y_key,
    //             &accounts.contract_key,
    //             &accounts.pool_mint_key,
    //             &accounts.pool_wallet_x_key,
    //             &accounts.authority_key,
    //             1,
    //         ),
    //         vec![
    //             &mut Account::default(),
    //             &mut accounts.user_wallet_x_account,
    //             &mut accounts.user_wallet_y_account,
    //             &mut accounts.contract_account,
    //             &mut accounts.pool_mint_account,
    //             &mut accounts.pool_wallet_x_account,
    //             &mut Account::default(),
    //             &mut Account::default(),
    //         ],
    //     )
    //     .unwrap();
    //
    //     do_process_instruction(
    //         CrateInstruction::change_y_to_x(
    //             &CONTRACT_PROGRAM_ID,
    //             &accounts.user_wallets_authority_key,
    //             &accounts.user_wallet_x_key,
    //             &accounts.user_wallet_y_key,
    //             &accounts.contract_key,
    //             &accounts.pool_mint_key,
    //             &accounts.pool_wallet_x_key,
    //             &accounts.authority_key,
    //             1,
    //         ),
    //         vec![
    //             &mut Account::default(),
    //             &mut accounts.user_wallet_x_account,
    //             &mut accounts.user_wallet_y_account,
    //             &mut accounts.contract_account,
    //             &mut accounts.pool_mint_account,
    //             &mut accounts.pool_wallet_x_account,
    //             &mut Account::default(),
    //             &mut Account::default(),
    //         ],
    //     )
    //     .unwrap();
    // }
}
