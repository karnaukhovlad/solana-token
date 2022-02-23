mod utils;

use crate::contract::TestContract;
use crate::token_x::TokenX;
use crate::user::User;
use solana_program_test::*;
use solana_sdk::signature::Signer;
use utils::*;

async fn setup() -> (ProgramTestContext, TestContract, TokenX, User) {
    let mut context = program_test().start_with_context().await;

    let test_contract = TestContract::new();
    test_contract.create(&mut context).await.unwrap();

    let token_x = TokenX::new();
    token_x.init(&mut context).await.unwrap();

    let user = User::new();
    user.init(&mut context, &token_x, &test_contract)
        .await
        .unwrap();

    (context, test_contract, token_x, user)
}

#[tokio::test]
async fn success() {
    let (mut context, test_contract, token_x, user) = setup().await;

    let amount = 100;

    token_x
        .mint_to(
            &mut context,
            &user.user_wallet_x.pubkey(),
            &token_x.owner,
            amount,
        )
        .await
        .unwrap();
    test_contract
        .change_x_to_y(&mut context, &user, amount)
        .await
        .unwrap();
}
