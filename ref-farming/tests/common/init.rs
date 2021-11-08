
use super::utils::to_va;

use near_sdk::{AccountId};
use near_sdk_sim::{call, deploy, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS};
use near_sdk::serde_json::json;

// use near_sdk_sim::transaction::ExecutionStatus;

use test_token::ContractContract as TestToken;
use ref_farming::{ContractContract as Farming};
use test_nft::{ContractContract as TestNFT, Contract};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TEST_TOKEN_WASM_BYTES => "../res/test_token.wasm",
    EXCHANGE_WASM_BYTES => "../res/ref_exchange_release.wasm",
    FARM_WASM_BYTES => "../res/ref_farming_release.wasm",
    NFT_WASM_BYTES => "../res/test_nft.wasm"
}

pub fn deploy_farming(root: &UserAccount, farming_id: AccountId, owner_id: AccountId) -> ContractAccount<Farming> {
    let farming = deploy!(
        contract: Farming,
        contract_id: farming_id,
        bytes: &FARM_WASM_BYTES,
        signer_account: root,
        init_method: new(to_va(owner_id))
    );
    farming
}

pub fn deploy_pool(root: &UserAccount, contract_id: AccountId, owner_id: AccountId) -> UserAccount {

    let pool = root.deploy(
        &EXCHANGE_WASM_BYTES,
        contract_id.clone(),
        to_yocto("100")
    );

    pool.call(
        contract_id,
        "new",
        &json!({
            "owner_id": owner_id,
            "exchange_fee": 4,
            "referral_fee": 1
        }).to_string().into_bytes(),
        DEFAULT_GAS / 2,
        0
    );

    pool
}

pub fn deploy_token(
    root: &UserAccount,
    token_id: AccountId,
    accounts_to_register: Vec<AccountId>,
) -> ContractAccount<TestToken> {
    let t = deploy!(
        contract: TestToken,
        contract_id: token_id,
        bytes: &TEST_TOKEN_WASM_BYTES,
        signer_account: root
    );
    call!(root, t.new()).assert_success();
    // call!(
    //     root,
    //     t.mint(to_va(root.account_id.clone()), to_yocto("10000").into())
    // )
    // .assert_success();
    for account_id in accounts_to_register {
        call!(
            root,
            t.storage_deposit(Some(to_va(account_id)), None),
            deposit = to_yocto("1")
        )
        .assert_success();
    }
    t
}

pub fn deploy_nft_contract(
    root: &UserAccount,
    nft_token_id: AccountId,
    farmer_id: AccountId,
) -> (ContractAccount<TestNFT>) {
    // uses default values for deposit and gas
    let nft = deploy!(
        // Contract Proxy
        contract: TestNFT,
        // Contract account id
        contract_id: nft_token_id,
        // Bytes of contract
        bytes: &NFT_WASM_BYTES,
        // User deploying the contract,
        signer_account: root,
        // init method
        init_method: new_default_meta(
            root.valid_account_id()
        )
    );

    for i in 0..10 {
        call!(
            root,
            nft.nft_mint(
                i.to_string(),
                to_va(farmer_id.clone()),
                TokenMetadata {
                    title: Some("Olympus Mons".into()),
                    description: Some("The tallest mountain in the charted solar system".into()),
                    media: None,
                    media_hash: None,
                    copies: Some(1u64),
                    issued_at: None,
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: None,
                    reference_hash: None,
                }
            ),
            deposit = 7000000000000000000000
        );
    }
    nft
}