use crate::{create_mint, create_token_account, TokenX, User};
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport,
};
use solana_token::{find_program_address, id, instruction};

#[derive(Debug)]
pub struct TestContract {
    pub contract: Keypair,
    pub pool_mint: Keypair,
    pub program_authority_id: Pubkey,
}

impl TestContract {
    pub fn new() -> Self {
        let contract = Keypair::new();
        let (program_authority_id, _) = find_program_address(&id(), &contract.pubkey());

        Self {
            contract,
            pool_mint: Keypair::new(),
            program_authority_id,
        }
    }

    pub async fn create(&self, context: &mut ProgramTestContext) -> transport::Result<()> {
        create_mint(context, &self.pool_mint, &context.payer.pubkey()).await?;

        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::create_account(
                &context.payer.pubkey(),
                &self.contract.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                0,
                &id(),
            )],
            Some(&context.payer.pubkey()),
            /// Проверить выполнится ли без owner keypair?
            &[&context.payer, &self.contract],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn change_x_to_y(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
        amount: u64,
    ) -> transport::Result<()> {
        let (pool_wallet_x, seed) = find_program_address(&id(), &user.user_wallet_x.pubkey());
        let (program_authority, seed) = find_program_address(&id(), &self.contract.pubkey());

        let tx = Transaction::new_signed_with_payer(
            &[instruction::change_x_to_y(
                &id(),
                &user.account.pubkey(),
                &user.user_wallet_x.pubkey(),
                &user.user_wallet_y.pubkey(),
                &self.contract.pubkey(),
                &self.pool_mint.pubkey(),
                &pool_wallet_x,
                &program_authority,
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &user.account],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
    //
    // pub async fn withdraw(
    //     &self,
    //     context: &mut ProgramTestContext,
    //     test_pool_market: &TestPoolMarket,
    //     user: &User,
    //     amount: u64,
    // ) -> transport::Result<()> {
    //     let tx = Transaction::new_signed_with_payer(
    //         &[instruction::withdraw(
    //             &id(),
    //             &test_pool_market.pool_market.pubkey(),
    //             &self.pool_pubkey,
    //             &user.destination,
    //             &user.source,
    //             &self.token_account.pubkey(),
    //             &self.pool_mint.pubkey(),
    //             &user.pubkey(),
    //             amount,
    //         )],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer, &user.owner],
    //         context.last_blockhash,
    //     );
    //
    //     context.banks_client.process_transaction(tx).await
    // }
    //
    // pub async fn borrow(
    //     &self,
    //     context: &mut ProgramTestContext,
    //     test_pool_market: &TestPoolMarket,
    //     test_pool_borrow_authority: &TestPoolBorrowAuthority,
    //     borrow_authority: &Keypair,
    //     destination: &Pubkey,
    //     amount: u64,
    // ) -> transport::Result<()> {
    //     let tx = Transaction::new_signed_with_payer(
    //         &[instruction::borrow(
    //             &id(),
    //             &test_pool_market.pool_market.pubkey(),
    //             &self.pool_pubkey,
    //             &test_pool_borrow_authority.pool_borrow_authority_pubkey,
    //             destination,
    //             &self.token_account.pubkey(),
    //             &borrow_authority.pubkey(),
    //             amount,
    //         )],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer, &borrow_authority],
    //         context.last_blockhash,
    //     );
    //
    //     context.banks_client.process_transaction(tx).await
    // }
    //
    // pub async fn repay(
    //     &self,
    //     context: &mut ProgramTestContext,
    //     test_pool_market: &TestPoolMarket,
    //     test_pool_borrow_authority: &TestPoolBorrowAuthority,
    //     user: &User,
    //     amount: u64,
    //     interest_amount: u64,
    // ) -> transport::Result<()> {
    //     let tx = Transaction::new_signed_with_payer(
    //         &[instruction::repay(
    //             &id(),
    //             &test_pool_market.pool_market.pubkey(),
    //             &self.pool_pubkey,
    //             &test_pool_borrow_authority.pool_borrow_authority_pubkey,
    //             &user.source,
    //             &self.token_account.pubkey(),
    //             &user.pubkey(),
    //             amount,
    //             interest_amount,
    //         )],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer, &user.owner],
    //         context.last_blockhash,
    //     );
    //
    //     context.banks_client.process_transaction(tx).await
    // }
}
