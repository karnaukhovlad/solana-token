use crate::{create_mint, create_token_account, mint_tokens};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::transport;

#[derive(Debug)]
pub struct TokenX {
    pub owner: Keypair,
    pub mint: Keypair,
}

impl TokenX {
    pub fn new() -> Self {
        Self {
            owner: Keypair::new(),
            mint: Keypair::new(),
        }
    }

    pub async fn init(&self, context: &mut ProgramTestContext) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::create_account(
                &context.payer.pubkey(),
                &self.owner.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            )],
            Some(&context.payer.pubkey()),
            /// Проверить выполнится ли без owner keypair?
            &[&context.payer, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await?;

        create_mint(context, &self.mint, &self.owner.pubkey()).await
    }

    pub async fn create_account(
        &self,
        context: &mut ProgramTestContext,
        owner: &Pubkey,
    ) -> transport::Result<Keypair> {
        let new_account = Keypair::new();
        create_token_account(context, &new_account, &self.mint.pubkey(), owner).await?;
        Ok(new_account)
    }

    pub async fn mint_to(
        &self,
        context: &mut ProgramTestContext,
        destination: &Pubkey,
        authority: &Keypair,
        amount: u64,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[spl_token::instruction::mint_to(
                &spl_token::id(),
                &self.mint.pubkey(),
                destination,
                &authority.pubkey(),
                &[],
                amount,
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
