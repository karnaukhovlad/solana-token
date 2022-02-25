use crate::{create_mint, TokenX, User};
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport,
};
use solana_token::{find_program_address, id, instruction};

#[derive(Debug)]
pub struct TestContract {
    pub pool_mint: Keypair,
    pub mint_authority: Pubkey,
}

impl TestContract {
    pub fn new() -> Self {
        let pool_mint = Keypair::new();
        let (mint_authority, _) = find_program_address(&id(), &pool_mint.pubkey());
        println!("TestContract: pool mint: {}", pool_mint.pubkey());
        println!("TestContract: pool mint authority: {}", mint_authority);
        Self {
            pool_mint,
            mint_authority,
        }
    }

    pub async fn create(&self, context: &mut ProgramTestContext) -> transport::Result<()> {
        create_mint(context, &self.pool_mint, &self.mint_authority).await
    }

    pub async fn change_x_to_y(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
        token_x: &TokenX,
        amount: u64,
    ) -> transport::Result<()> {
        println!("Payer {}", &context.payer.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[instruction::change_x_to_y(
                &id(),
                &user.account.pubkey(),
                &user.user_wallet_x.pubkey(),
                &token_x.mint.pubkey(),
                &user.user_wallet_y.pubkey(),
                &self.pool_mint.pubkey(),
                amount,
            )],
            Some(&user.account.pubkey()),
            &[&user.account],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn change_y_to_x(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
        _token_x: &TokenX,
        amount: u64,
    ) -> transport::Result<()> {
        println!("Payer {}", &context.payer.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[instruction::change_y_to_x(
                &id(),
                &user.account.pubkey(),
                &user.user_wallet_x.pubkey(),
                &user.user_wallet_y.pubkey(),
                &self.pool_mint.pubkey(),
                amount,
            )],
            Some(&user.account.pubkey()),
            &[&user.account],
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
