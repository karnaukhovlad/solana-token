use crate::token_x::TokenX;
use crate::{create_mint, create_token_account, TestContract};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
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
        Self {
            account: Keypair::new(),
            user_wallet_x: Keypair::new(),
            user_wallet_y: Keypair::new(),
        }
    }

    pub async fn init(
        &self,
        context: &mut ProgramTestContext,
        token_x: &TokenX,
        contract: &TestContract,
    ) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::create_account(
                &context.payer.pubkey(),
                &self.account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            )],
            Some(&context.payer.pubkey()),
            /// Проверить выполнится ли без owner keypair?
            &[&context.payer, &self.account],
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
