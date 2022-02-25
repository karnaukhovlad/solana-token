mod utils;

use crate::contract::TestContract;
use crate::token_x::TokenX;
use crate::user::User;
use solana_program_test::*;
use solana_sdk::signature::Signer;
use solana_token::find_program_address;
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

    let user_wallet_x = user.user_wallet_x.pubkey();
    let user_wallet_y = user.user_wallet_y.pubkey();
    let (pool_wallet_x, _) =
        find_program_address(&solana_token::id(), &user.user_wallet_x.pubkey());
    assert_eq!(get_token_balance(&mut context, &user_wallet_x).await, 0);
    token_x
        .mint_to(
            &mut context,
            &user.user_wallet_x.pubkey(),
            &token_x.owner,
            amount,
        )
        .await
        .unwrap();
    assert_eq!(
        get_token_balance(&mut context, &user_wallet_x).await,
        amount
    );
    assert_eq!(get_token_balance(&mut context, &user_wallet_y).await, 0);
    test_contract
        .change_x_to_y(&mut context, &user, &token_x, amount)
        .await
        .unwrap();
    assert_eq!(get_token_balance(&mut context, &user_wallet_x).await, 0);
    assert_eq!(
        get_token_balance(&mut context, &pool_wallet_x).await,
        amount
    );
    assert_eq!(
        get_token_balance(&mut context, &user_wallet_y).await,
        amount
    );
    test_contract
        .change_y_to_x(&mut context, &user, &token_x, amount)
        .await
        .unwrap();
    assert_eq!(
        get_token_balance(&mut context, &user_wallet_x).await,
        amount
    );
    assert_eq!(get_token_balance(&mut context, &pool_wallet_x).await, 0);
    assert_eq!(get_token_balance(&mut context, &user_wallet_y).await, 0);
}
