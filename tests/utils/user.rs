use crate::token_x::TokenX;
use crate::{create_token_account, TestContract};
use solana_program::system_instruction;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::transport;

#[derive(Debug)]
pub struct User {
    pub account: Keypair,
    pub user_wallet_x: Keypair,
    pub user_wallet_y: Keypair,
}

impl User {
    pub fn new() -> Self {
        let account = Keypair::new();
        let user_wallet_x = Keypair::new();
        let user_wallet_y = Keypair::new();
        println!("User: account {}", account.pubkey());
        println!("User: wallet x {}", user_wallet_x.pubkey());
        println!("User: wallet y {}", user_wallet_y.pubkey());
        Self {
            account,
            user_wallet_x,
            user_wallet_y,
        }
    }

    pub async fn init(
        &self,
        context: &mut ProgramTestContext,
        token_x: &TokenX,
        contract: &TestContract,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &context.payer.pubkey(),
                &self.account.pubkey(),
                999999999,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await?;

        create_token_account(
            context,
            &self.user_wallet_x,
            &token_x.mint.pubkey(),
            &self.account.pubkey(),
        )
        .await?;
        create_token_account(
            context,
            &self.user_wallet_y,
            &contract.pool_mint.pubkey(),
            &self.account.pubkey(),
        )
        .await
    }
}
