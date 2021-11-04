/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
 */
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::core::{NonFungibleTokenCore, NonFungibleTokenResolver};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, assert_one_yocto, serde_json::json,
    Gas, ext_contract, Balance
};

use std::collections::HashMap;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 30_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;
const NO_DEPOSIT: Balance = 0;


#[ext_contract(ext_self)]
trait NFTResolver {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}

#[ext_contract(ext_receiver)]
pub trait NonFungibleTokenReceiver {
    /// Returns true if token should be returned to `sender_id`
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Example NEAR non-fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        self.tokens.mint(token_id, receiver_id, Some(token_metadata))
    }



    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: String,
        approval_id: u64,
        balance: U128,
        max_len_payout: u32,
    ) -> HashMap<AccountId, U128> {
        assert_one_yocto();

        let owner_id = self.tokens.owner_by_id.get(&token_id).expect("Token id does not exist");
        self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), Some(approval_id), None);

        env::log(
            json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": owner_id,
                    "receiver_id": receiver_id,
                }
            })
                .to_string()
                .as_bytes(),
        );

        let mut result: HashMap<AccountId, U128> = HashMap::new();
        result.insert(owner_id, balance);
        result
    }



}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        env::log(
            json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id.clone(),
                    "sender_id": sender_id,
                    "receiver_id": receiver_id.clone()
                }
            })
                .to_string()
                .as_bytes(),
        );

        let (old_owner, old_approvals) =
            self.tokens.internal_transfer(&sender_id, receiver_id.as_ref(), &token_id, approval_id, memo);
        // Initiating receiver's call and the callback
        ext_receiver::nft_on_transfer(
            sender_id.clone(),
            old_owner.clone(),
            token_id.clone(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
            .then(ext_self::nft_resolve_transfer(
                old_owner,
                receiver_id.into(),
                token_id,
                old_approvals,
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_RESOLVE_TRANSFER,
            ))
            .into()
    }

    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let receiver_id_str = receiver_id.to_string();
        let sender_id = env::predecessor_account_id();
        self.tokens
            .nft_transfer(receiver_id, token_id.clone(), approval_id, memo);
        env::log(
            json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": sender_id,
                    "receiver_id": receiver_id_str
                }
            })
                .to_string()
                .as_bytes(),
        );
    }

    fn nft_token(self, token_id: TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id)
    }

    #[payable]
    fn mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: ValidAccountId,
        token_metadata: Option<TokenMetadata>,
    ) -> Token {
        self.tokens.mint(token_id, token_owner_id, token_metadata)
    }
}

//near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        let resp: bool = self.tokens.nft_resolve_transfer(
            previous_owner_id.clone(),
            receiver_id.clone(),
            token_id.clone(),
            approved_account_ids,
        );

        if resp {
            env::log(
                json!({
                "type": "nft_transfer",
                "params": {
                    "token_id": token_id,
                    "sender_id": previous_owner_id,
                    "receiver_id": receiver_id,
                }
            })
                    .to_string()
                    .as_bytes(),
            );
        }

        resp
    }
}