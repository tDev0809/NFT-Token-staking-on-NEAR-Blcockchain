
use std::convert::TryInto;
use near_sdk::json_types::{U128};
use near_sdk::{AccountId, Balance, PromiseResult};

use crate::utils::{
    assert_one_yocto, ext_multi_fungible_token, ext_fungible_token, ext_non_fungible_token,
    ext_self, wrap_mft_token_id, parse_seed_id, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER,
    GAS_FOR_NFT_TRANSFER
};
use crate::errors::*;
use crate::farm_seed::SeedType;
use crate::*;
use crate::simple_farm::{NFTTokenId, ContractNFTTokenId};
use std::collections::HashMap;
use near_sdk::collections::UnorderedSet;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn withdraw_nft(&mut self, seed_id: SeedId, nft_contract_id: String, nft_token_id: NFTTokenId) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let nft_token_id = self.internal_nft_withdraw(&seed_id, &sender_id, &nft_contract_id, &nft_token_id);

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

        let amount: Balance = amount.into();

        // update inner state
        let seed_type = self.internal_seed_withdraw(&seed_id, &sender_id, amount);

        match seed_type {
            SeedType::FT => {
                ext_fungible_token::ft_transfer(
                    sender_id.clone().try_into().unwrap(),
                    amount.into(),
                    None,
                    &seed_id,
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
            }
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

                //  farmer first
                farmer.get_ref_mut().add_nft(&seed_id, &nft_contract_id, &nft_token_id);

                let amount_before_multiplier: u128 = *farmer.get_ref().seeds_after_multiplier.get(&seed_id).unwrap_or(&0u128);
                let amount_after_multiplier: u128 = self.calculate_amount_after_multiplier(
                    &farm_seed.get_ref().nft_multiplier,
                    farmer.get_ref().nft_seeds.get(seed_id.as_str()),
                    farmer.get_ref().seeds.get(seed_id.as_str())
                );

                farmer.get_ref_mut().set_seed_after_multiplier(&seed_id, amount_after_multiplier);
                self.data_mut().farmers.insert(&sender_id, &farmer);


                // **** update seed (new version)
                farm_seed.get_ref_mut().sub_amount(amount_before_multiplier);
                farm_seed.get_ref_mut().add_amount(amount_after_multiplier);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);
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
                let mut farm_seed = self.get_seed(&seed_id);
                let mut farmer = self.get_farmer(&sender_id);

                farm_seed.get_ref_mut().seed_type = SeedType::FT;
                farm_seed.get_ref_mut().add_amount(amount);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);
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
                let mut farm_seed = self.get_seed(&seed_id);
                let mut farmer = self.get_farmer(&sender_id);

                farm_seed.get_ref_mut().seed_type = SeedType::MFT;
                farm_seed.get_ref_mut().add_amount(amount);
                farmer.get_ref_mut().add_seed(&seed_id, amount);
                self.data_mut().seeds.insert(&seed_id, &farm_seed);
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
    pub(crate) fn get_seed(&self, seed_id: &String) -> VersionedFarmSeed {
        let orig = self.data().seeds.get(seed_id).expect(&format!("{}", ERR31_SEED_NOT_EXIST));
        if orig.need_upgrade() {
            orig.upgrade()
        } else {
            orig
        } 
    }

    #[inline]
    pub(crate) fn get_seed_wrapped(&self, seed_id: &String) -> Option<VersionedFarmSeed> {
        if let Some(farm_seed) = self.data().seeds.get(seed_id) {
            if farm_seed.need_upgrade() {
                Some(farm_seed.upgrade())
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

        // **** update seed (new version)
        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);
        farmer.get_ref_mut().add_seed(&seed_id, amount);

        let amount_before_multiplier: u128 = *farmer.get_ref().seeds_after_multiplier.get(seed_id).unwrap_or(&0u128);
        let amount_after_multiplier: u128 = self.calculate_amount_after_multiplier(
            &farm_seed.get_ref().nft_multiplier,
            farmer.get_ref().nft_seeds.get(seed_id.as_str()),
            farmer.get_ref().seeds.get(seed_id.as_str())
        );

        farm_seed.get_ref_mut().seed_type = seed_type;
        farm_seed.get_ref_mut().sub_amount(amount_before_multiplier);
        farm_seed.get_ref_mut().add_amount(amount_after_multiplier);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);

        farmer.get_ref_mut().set_seed_after_multiplier(&seed_id, amount_after_multiplier);
        self.data_mut().farmers.insert(sender_id, &farmer);
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

        let amount_before_multiplier: u128 = *farmer.get_ref().seeds_after_multiplier.get(seed_id).unwrap_or(&0u128);
        let amount_after_multiplier: u128 = self.calculate_amount_after_multiplier(
            &farm_seed.get_ref().nft_multiplier,
            farmer.get_ref().nft_seeds.get(seed_id.as_str()),
            farmer.get_ref().seeds.get(seed_id.as_str())
        );

        farm_seed.get_ref_mut().sub_amount(amount_before_multiplier);
        let _seed_remain = farm_seed.get_ref_mut().sub_amount(amount_after_multiplier);

        if farmer_seed_remain == 0 {
            // remove farmer rps of relative farm
            for farm_id in farm_seed.get_ref().farms.iter() {
                farmer.get_ref_mut().remove_rps(farm_id);
            }
        }
        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);
        farm_seed.get_ref().seed_type.clone()
    }

    pub(crate) fn internal_nft_deposit(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String,
    ) {

        // first claim all reward of the user for this seed farms
        // to update user reward_per_seed in each farm
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);


        let mut farmer = self.get_farmer(sender_id);
        let mut farm_seed = self.get_seed(seed_id);

        //  farmer first
        farmer.get_ref_mut().add_nft(seed_id, nft_contract_id, nft_token_id);

        let amount_before_multiplier: u128 = *farmer.get_ref().seeds_after_multiplier.get(seed_id).unwrap_or(&0u128);
        let amount_after_multiplier: u128 = self.calculate_amount_after_multiplier(
            &farm_seed.get_ref().nft_multiplier,
            farmer.get_ref().nft_seeds.get(seed_id.as_str()),
            farmer.get_ref().seeds.get(seed_id.as_str())
        );

        farmer.get_ref_mut().set_seed_after_multiplier(&seed_id, amount_after_multiplier);
        self.data_mut().farmers.insert(sender_id, &farmer);


        // **** update seed (new version)
        farm_seed.get_ref_mut().sub_amount(amount_before_multiplier);
        farm_seed.get_ref_mut().add_amount(amount_after_multiplier);
        self.data_mut().seeds.insert(&seed_id, &farm_seed);
    }

    fn internal_nft_withdraw(
        &mut self,
        seed_id: &String,
        sender_id: &AccountId,
        nft_contract_id: &String,
        nft_token_id: &String
    ) -> String {
        self.internal_claim_user_reward_by_seed_id(sender_id, seed_id);

        let mut farm_seed = self.get_seed(seed_id);
        let mut farmer = self.get_farmer(sender_id);

        // sub nft
        let nft_token_id = farmer.get_ref_mut().sub_nft(seed_id, nft_contract_id, nft_token_id).unwrap();

        // calculate farm_seed after multiplier get removed
        let amount_before_multiplier: u128 = *farmer.get_ref().seeds_after_multiplier.get(seed_id).unwrap_or(&0u128);
        let amount_after_multiplier: u128 = self.calculate_amount_after_multiplier(
            &farm_seed.get_ref().nft_multiplier,
            farmer.get_ref().nft_seeds.get(seed_id.as_str()),
            farmer.get_ref().seeds.get(seed_id.as_str())
        );

        farmer.get_ref_mut().set_seed_after_multiplier(&seed_id, amount_after_multiplier);
        farm_seed.get_ref_mut().sub_amount(amount_before_multiplier);
        let _seed_remain = farm_seed.get_ref_mut().sub_amount(amount_after_multiplier);

        self.data_mut().farmers.insert(sender_id, &farmer);
        self.data_mut().seeds.insert(seed_id, &farm_seed);
        nft_token_id
    }

    pub fn calculate_amount_after_multiplier(
        &self,
        nft_multiplier: &Option<HashMap<String, u32>>,
        nft_seeds: Option<&UnorderedSet<ContractNFTTokenId>>,
        ft_seed_balance: Option<&Balance>,
    ) -> u128 {
        if nft_multiplier.is_none() {
            return *ft_seed_balance.unwrap()
        }
        // split x.paras.near@1:1
        // to "x.paras.near@1", ":1"
        let mut multiplier: u128 = 0;
        if let Some(nft_multiplier) = nft_multiplier {
            nft_seeds
                .unwrap()
                .iter()
                .for_each(
                    |x: ContractNFTTokenId| {
                        let contract_token_series_id_split: Vec<&str> = x.split(':').collect();
                        let multiply = *nft_multiplier.get(&contract_token_series_id_split[0].to_string()).unwrap_or(&0);
                        multiplier += multiply as u128;
                    }
                );
        }


        let amount_after_multiplier : u128 = ft_seed_balance.unwrap() + ft_seed_balance.unwrap() / 10000 * multiplier;
        return amount_after_multiplier;
    }
}
