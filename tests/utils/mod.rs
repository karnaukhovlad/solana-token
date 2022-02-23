pub mod contract;
pub mod token_x;
pub mod user;

use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::transport;
use solana_token::{id, processor};

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "solana-token",
        id(),
        processor!(processor::Processor::process_instruction),
    )
}

// pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
//     context
//         .banks_client
//         .get_account(*pubkey)
//         .await
//         .expect("account not found")
//         .expect("account empty")
// }
//
// pub async fn get_token_balance(context: &mut ProgramTestContext, pubkey: &Pubkey) -> u64 {
//     let account = get_account(context, pubkey).await;
//     let account_info: spl_token::state::Account =
//         spl_token::state::Account::unpack_from_slice(account.data.as_slice()).unwrap();
//
//     account_info.amount
// }

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    authority: &Pubkey,
) -> transport::Result<()> {
    println!("create_token_account");
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                authority,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Pubkey,
) -> transport::Result<()> {
    println!("create_mint");
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &authority,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
) -> transport::Result<()> {
    println!("mint_tokens");
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &context.payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
