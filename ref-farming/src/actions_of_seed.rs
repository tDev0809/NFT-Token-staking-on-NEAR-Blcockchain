
use std::convert::TryInto;
use near_sdk::json_types::{U128};
use near_sdk::{AccountId, Balance, PromiseResult};

use crate::utils::{assert_one_yocto, ext_multi_fungible_token, ext_fungible_token, ext_non_fungible_token, ext_self, wrap_mft_token_id, parse_seed_id, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, GAS_FOR_NFT_TRANSFER, FT_INDEX_TAG, get_nft_balance_equivalent};
use crate::errors::*;
use crate::farm_seed::SeedType;
use crate::*;
use crate::simple_farm::{NFTTokenId, ContractNFTTokenId};
use crate::utils::NFT_DELIMETER;
use std::collections::HashMap;

#[near_bindgen]
impl Contract {

    pub fn force_upgrade_seed(&mut self, seed_id: SeedId) {
        self.assert_owner();
        let seed = self.get_seed_and_upgrade(&seed_id);
        self.data_mut().seeds.insert(&seed_id, &seed);
    }

    #[payable]
    pub fn withdraw_nft(&mut self, seed_id: SeedId, nft_contract_id: String, nft_token_id: NFTTokenId) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        self.internal_nft_withdraw(&seed_id, &sender_id, &nft_contract_id, &nft_token_id);

        // transfer nft back to the owner
        ext_non_fungible_token::nft_transfer(
            sender_id.clone(),
            nft_token_id.clone(),
            None,
            None,
            &nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER
        )
        .then(ext_self::callback_post_withdraw_nft(
            seed_id,
            sender_id,
            nft_contract_id,
            nft_token_id,
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_TRANSFER
        ));
    }

    #[payable]
    pub fn withdraw_seed(&mut self, seed_id: SeedId, amount: U128) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let seed_contract_id: AccountId = seed_id.split(FT_INDEX_TAG).next().unwrap().to_string();
        let amount: Balance = amount.into();

        // update inner state
        let seed_type = self.internal_seed_withdraw(&seed_id, &sender_id, amount);

        match seed_type {
            SeedType::FT => {
                ext_fungible_token::ft_transfer(
                    sender_id.clone().try_into().unwrap(),
                    amount.into(),
                    None,
                    &seed_contract_id,
                    1,  // one yocto near
                    GAS_FOR_FT_TRANSFER,
                )
                    .then(ext_self::callback_post_withdraw_ft_seed(
                        seed_id,
                        sender_id,
                        amount.into(),
                        &env::current_account_id(),
                        0,
                        GAS_FOR_RESOLVE_TRANSFER,
                    ));
            },
            SeedType::NFT => {
                panic!("Use withdraw_nft for this");
            },
            SeedType::MFT => {
                let (receiver_id, token_id) = parse_seed_id(&seed_id);
                ext_multi_fungible_token::mft_transfer(
                    wrap_mft_token_id(&token_id),
                    sender_id.clone().try_into().unwrap(),
                    amount.into(),
                    None,
                    &receiver_id,
                    1,  // one yocto near
                    GAS_FOR_FT_TRANSFER,
                )
                    .then(ext_self::callback_post_withdraw_mft_seed(
                        seed_id,
                        sender_id,
                        amount.into(),
                        &env::current_account_id(),
                        0,
                        GAS_FOR_RESOLVE_TRANSFER,
                    ));
            }
        }
    }

    #[private]
    pub fn callback_post_withdraw_nft(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        nft_contract_id: String,
        nft_token_id: String
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} nft from {}, Callback failed.",
                        sender_id, nft_token_id, nft_contract_id
                    ).as_bytes()
                );

                // revert withdraw

                let mut farmer = self.get_farmer(&sender_id);
                let mut farm_seed = self.get_seed(&seed_id);

                let contract_nft_token_id : ContractNFTTokenId = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
                let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
                if let Some(nft_balance_equivalent) = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()) {
                    self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);

                    farmer.get_ref_mut().add_nft(&seed_id, contract_nft_token_id);

                    farmer.get_ref_mut().add_seed(&seed_id, nft_balance_equivalent);
                    self.data_mut().farmers.insert(&sender_id, &farmer);

                    // **** update seed (new version)
                    farm_seed.get_ref_mut().add_amount(nft_balance_equivalent);
                    self.data_mut().seeds.insert(&seed_id, &farm_seed);
                }
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} nft from {}, Succeed.",
                        sender_id, nft_token_id, nft_contract_id
                    ).as_bytes()
                );
            }
        }
    }
    #[private]
    pub fn callback_post_withdraw_ft_seed(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} ft seed with amount {}, Callback Failed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
                // revert withdraw, equal to deposit, claim reward to update user reward_per_seed
                self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
                // **** update seed (new version)
                let mut farm_seed = self.get_seed(&seed_id);
                farm_seed.get_ref_mut().add_amount(amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);

                let mut farmer = self.get_farmer(&sender_id);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().farmers.insert(&sender_id, &farmer);
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} ft seed with amount {}, Succeed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
            }
        };
    }

    #[private]
    pub fn callback_post_withdraw_mft_seed(
        &mut self,
        seed_id: SeedId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw {} mft seed with amount {}, Callback Failed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
                // revert withdraw, equal to deposit, claim reward to update user reward_per_seed

                self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
                // **** update seed (new version)
                let mut farm_seed = self.get_seed(&seed_id);
                farm_seed.get_ref_mut().add_amount(amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);

                let mut farmer = self.get_farmer(&sender_id);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().farmers.insert(&sender_id, &farmer);
            },
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw {} mft seed with amount {}, Succeed.",
                        sender_id, seed_id, amount,
                    )
                    .as_bytes(),
                );
            }
        };
    }
}


/// Internal methods implementation.
impl Contract {

    #[inline]
    pub(crate) fn get_seed_and_upgrade(&mut self, seed_id: &String) -> VersionedFarmSeed {
        let orig = self.data().seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));
        if orig.need_upgrade() {
            orig.upgrade(self)
        } else {
            orig
        }
    }

    #[inline]
    pub(crate) fn get_seed(&self, seed_id: &String) -> VersionedFarmSeed {
        let orig = self.data().seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));
        if orig.need_upgrade() {
            panic!("Need upgrade");
        } else {
            orig
        } 
    }

    #[inline]
    pub(crate) fn get_seed_wrapped(&self, seed_id: &String) -> Option<VersionedFarmSeed> {
        if let Some(farm_seed) = self.data().seeds.get(seed_id) {
            if farm_seed.need_upgrade() {
                panic!("Need upgrade");
            } else {
                Some(farm_seed)
            }
        } else {
            None
        }
    }

    pub(crate) fn internal_seed_deposit(
        &mut self, 
        seed_id: &String, 
        sender_id: &AccountId, 
        amount: Balance, 
        seed_type: SeedType) {

        // first claim all reward of the user for this seed farms
        // to update user reward_per_seed in each farm
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);

        let mut farmer = self.get_farmer(sender_id);

        // **** update seed (new version)
        farm_seed.get_ref_mut().add_amount(amount);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);

        farmer.get_ref_mut().add_seed(&seed_id, amount);
        self.data_mut().farmers.insert(sender_id, &farmer);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };
    }

    pub(crate) fn internal_seed_redeposit(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        is_deposit_seed_reward: bool,
    ) {
        self.internal_claim_user_reward_by_seed_id(&sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        let amount = if farmer.get_ref().rewards.get(seed_id).is_some() {
            farmer.get_ref_mut().sub_reward(&seed_id, 0)
        } else {
            0
        };

        // **** update seed (new version)
        farm_seed.get_ref_mut().add_amount(amount);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);

        farmer.get_ref_mut().add_seed(&seed_id, amount);
        self.data_mut().farmers.insert(sender_id, &farmer);

        let mut reward_tokens: Vec<AccountId> = vec![];

        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };
    }

    fn internal_seed_withdraw(
        &mut self, 
        seed_id: &SeedId, 
        sender_id: &AccountId, 
        amount: Balance) -> SeedType {

        // first claim all reward of the user for this seed farms
        // to update user reward_per_seed in each farm
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        // Then update user seed and total seed of this LPT
        let farmer_seed_remain = farmer.get_ref_mut().sub_seed(seed_id, amount);
        let _seed_remain = farm_seed.get_ref_mut().sub_amount(amount);

        if farmer_seed_remain == 0 {
            // remove farmer rps of relative farm
            for farm_id in farm_seed.get_ref().farms.iter() {
                farmer.get_ref_mut().remove_rps(farm_id);
            }
        }
        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };

        farm_seed.get_ref().seed_type.clone()
    }

    pub(crate) fn internal_nft_deposit(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String,
    ) -> bool {
        let mut farm_seed = self.get_seed(seed_id);

        assert_eq!(farm_seed.get_ref().seed_type, SeedType::NFT, "Cannot deposit NFT to this farm");

        // update farmer seed
        let contract_nft_token_id = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
        let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
        return if let Some(nft_balance_equivalent) = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()) {
            // first claim all reward of the user for this seed farms
            // to update user reward_per_seed in each farm
            self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);
            let mut farmer = self.get_farmer(sender_id);
            farmer.get_ref_mut().add_nft(seed_id, contract_nft_token_id);

            farmer.get_ref_mut().add_seed(seed_id, nft_balance_equivalent);
            self.data_mut().farmers.insert(sender_id, &farmer);

            // **** update seed (new version)
            farm_seed.get_ref_mut().add_amount(nft_balance_equivalent);
            self.data_mut().seeds.insert(&seed_id, &farm_seed);

            let mut reward_tokens: Vec<AccountId> = vec![];
            for farm_id in farm_seed.get_ref().farms.iter() {
                let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
                if !reward_tokens.contains(&reward_token) {
                    if farmer.get_ref().rewards.get(&reward_token).is_some() {
                        self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                    }
                    reward_tokens.push(reward_token);
                }
            };

            true
        } else {
            false
        }
    }

    fn internal_nft_withdraw(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String
    ) -> ContractNFTTokenId {
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        // sub nft
        let contract_nft_token_id : ContractNFTTokenId = format!("{}{}{}", nft_contract_id, NFT_DELIMETER, nft_token_id);
        farmer.get_ref_mut().sub_nft(seed_id, contract_nft_token_id.clone()).unwrap();
        let nft_balance = self.data().nft_balance_seeds.get(&seed_id).unwrap();
        let nft_balance_equivalent: Balance = get_nft_balance_equivalent(nft_balance, contract_nft_token_id.clone()).unwrap();

        let farmer_seed_remain = farmer.get_ref_mut().sub_seed(seed_id, nft_balance_equivalent);

        // calculate farm_seed after multiplier get removed
        farm_seed.get_ref_mut().sub_amount(nft_balance_equivalent);

        if farmer_seed_remain == 0 {
            // remove farmer rps of relative farm
            for farm_id in farm_seed.get_ref().farms.iter() {
                farmer.get_ref_mut().remove_rps(farm_id);
            }
        }

        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);

        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in farm_seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.private_withdraw_reward(reward_token.clone(), sender_id.to_string(), None);
                }
                reward_tokens.push(reward_token);
            }
        };

        contract_nft_token_id
    }

}
