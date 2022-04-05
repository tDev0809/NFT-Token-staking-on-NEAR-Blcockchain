
use std::convert::TryInto;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{assert_one_yocto, env, near_bindgen, AccountId, Balance, PromiseResult};

use crate::utils::{ext_fungible_token, ext_self, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, parse_farm_id};
use crate::errors::*;
use crate::*;
use uint::construct_uint;
use crate::token_receiver::TokenId;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[near_bindgen]
impl Contract {

    /// Clean invalid rps,
    /// return false if the rps is still valid.
    pub fn remove_user_rps_by_farm(&mut self, farm_id: FarmId) -> bool {
        let sender_id = env::predecessor_account_id();
        let mut farmer = self.get_farmer(&sender_id);
        let (seed_id, _) = parse_farm_id(&farm_id);
        let farm_seed = self.get_seed(&seed_id);
        if !farm_seed.get_ref().farms.contains(&farm_id) {
            farmer.get_ref_mut().remove_rps(&farm_id);
            self.data_mut().farmers.insert(&sender_id, &farmer);
            true
        } else {
            false
        }
    }

    pub fn claim_reward_by_farm(&mut self, farm_id: FarmId) {
        let sender_id = env::predecessor_account_id();
        self.internal_claim_user_reward_by_farm_id(&sender_id, &farm_id);
        self.assert_storage_usage(&sender_id);
    }

    pub fn claim_reward_by_seed(&mut self, seed_id: SeedId) {
        let sender_id = env::predecessor_account_id();
        self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
        self.assert_storage_usage(&sender_id);
    }

    pub fn claim_reward_by_seed_and_deposit(&mut self, seed_id: SeedId, seed_id_deposit: SeedId, is_deposit_seed_reward: bool) {
        let sender_id = env::predecessor_account_id();
        // only claim active farm with seed_id_deposit as its reward

        let seed = self.get_seed(&seed_id);

        for farm_id in seed.get_ref().farms.iter() {
            let farm = self.get_farm(farm_id.to_string()).unwrap();
            if farm.farm_status == "Running" && farm.reward_token == seed_id_deposit {
                self.internal_claim_user_reward_by_farm_id(&sender_id, farm_id);
            }
        }

        self.assert_storage_usage(&sender_id);

        self.internal_seed_redeposit(&seed_id_deposit, &sender_id, is_deposit_seed_reward);
    }

    pub fn claim_reward_by_all_seed_and_deposit(&mut self, seed_id_deposit: SeedId) {
        let sender_id = env::predecessor_account_id();
        let farmer = self.get_farmer(&sender_id);
        for (seed_id, _) in farmer.get_ref().seeds.iter() {
            let seed = self.get_seed(&seed_id);
            for farm_id in seed.get_ref().farms.iter() {
                let farm = self.get_farm(farm_id.to_string()).unwrap();
                if farm.farm_status == "Running" && farm.reward_token == seed_id_deposit {
                    self.internal_claim_user_reward_by_farm_id(&sender_id, &farm_id);
                }
            }
        }
        self.assert_storage_usage(&sender_id);

        self.internal_seed_redeposit(&seed_id_deposit, &sender_id, true);
    }

    #[payable]
    pub fn claim_reward_by_farm_and_withdraw(&mut self, farm_id: FarmId) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.internal_claim_user_reward_by_farm_id(&sender_id, &farm_id);
        self.assert_storage_usage(&sender_id);

        let token_id = self.get_farm(farm_id).unwrap().reward_token;
        self.internal_withdraw_reward(token_id, None);
    }

    #[payable]
    pub fn claim_reward_by_seed_and_withdraw(&mut self, seed_id: SeedId) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.internal_claim_user_reward_by_seed_id(&sender_id, &seed_id);
        self.assert_storage_usage(&sender_id);

        let farmer = self.get_farmer(&sender_id);

        let seed = self.data().seeds.get(&seed_id).unwrap();
        let mut reward_tokens: Vec<AccountId> = vec![];
        for farm_id in seed.get_ref().farms.iter() {
            let reward_token = self.data().farms.get(farm_id).unwrap().get_reward_token();
            if !reward_tokens.contains(&reward_token) {
                if farmer.get_ref().rewards.get(&reward_token).is_some() {
                    self.internal_withdraw_reward(reward_token.clone(), None);
                }
                reward_tokens.push(reward_token);
            }
        };
    }

    /// Withdraws given reward token of given user.
    #[payable]
    pub fn withdraw_reward(&mut self, token_id: ValidAccountId, amount: Option<U128>) {
        assert_one_yocto();

        self.internal_withdraw_reward(token_id.to_string(), amount);
    }

    #[private]
    pub fn private_withdraw_reward(&mut self, token_id: AccountId, sender_id: AccountId, amount: Option<U128>) {
        self.internal_execute_withdraw_reward(token_id, sender_id, amount);
    }

    fn internal_withdraw_reward(&mut self, token_id: AccountId, amount: Option<U128>) {
        let sender_id = env::predecessor_account_id();
        self.internal_execute_withdraw_reward(token_id, sender_id, amount);
    }

    fn internal_execute_withdraw_reward(&mut self, token_id: AccountId, sender_id: AccountId, amount: Option<U128>) {
        let token_id: AccountId = token_id.into();
        let amount: u128 = amount.unwrap_or(U128(0)).into();
        let mut farmer = self.get_farmer(&sender_id);

        // Note: subtraction, will be reverted if the promise fails.
        let amount = farmer.get_ref_mut().sub_reward(&token_id, amount);
        self.data_mut().farmers.insert(&sender_id, &farmer);
        if amount != 0 {
            ext_fungible_token::ft_transfer(
                sender_id.clone().try_into().unwrap(),
                amount.into(),
                None,
                &token_id,
                1,
                GAS_FOR_FT_TRANSFER,
            )
                .then(ext_self::callback_post_withdraw_reward(
                    token_id,
                    sender_id,
                    amount.into(),
                    &env::current_account_id(),
                    0,
                    GAS_FOR_RESOLVE_TRANSFER,
                ));
        }
    }

    #[private]
    pub fn callback_post_withdraw_reward(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR25_CALLBACK_POST_WITHDRAW_INVALID
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                env::log(
                    format!(
                        "{} withdraw reward {} amount {}, Succeed.",
                        sender_id, token_id, amount.0,
                    )
                    .as_bytes(),
                );
            }
            PromiseResult::Failed => {
                env::log(
                    format!(
                        "{} withdraw reward {} amount {}, Callback Failed.",
                        sender_id, token_id, amount.0,
                    )
                    .as_bytes(),
                );
                // This reverts the changes from withdraw function.
                let mut farmer = self.get_farmer(&sender_id);
                farmer.get_ref_mut().add_reward(&token_id, amount.0);
                self.data_mut().farmers.insert(&sender_id, &farmer);
            }
        };
    }
}

fn claim_user_reward_from_farm(
    farm: &mut Farm, 
    farmer: &mut Farmer, 
    total_seeds: &Balance,
    silent: bool,
) {
    let user_seeds = farmer.seeds.get(&farm.get_seed_id()).unwrap_or(&0_u128);
    let user_rps = farmer.get_rps(&farm.get_farm_id());
    let (new_user_rps, reward_amount) = farm.claim_user_reward(&user_rps, user_seeds, total_seeds, silent);
    if !silent {
        env::log(
            format!(
                "user_rps@{} increased to {}",
                farm.get_farm_id(), U256::from_little_endian(&new_user_rps),
            )
            .as_bytes(),
        );
    }
        
    farmer.set_rps(&farm.get_farm_id(), new_user_rps);
    if reward_amount > 0 {
        farmer.add_reward(&farm.get_reward_token(), reward_amount);
        if !silent {
            env::log(
                format!(
                    "claimed {} {} as reward from {}",
                    reward_amount, farm.get_reward_token() , farm.get_farm_id(),
                )
                .as_bytes(),
            );
        }
    }
}

impl Contract {

    pub(crate) fn internal_claim_user_reward_by_seed_id(
        &mut self, 
        sender_id: &AccountId,
        seed_id: &SeedId) {
        let mut farmer = self.get_farmer(sender_id);
        if let Some(mut farm_seed) = self.get_seed_wrapped(seed_id) {
            let amount = farm_seed.get_ref().amount;
            for farm_id in &mut farm_seed.get_ref_mut().farms.iter() {
                let mut farm = self.data().farms.get(farm_id).unwrap();
                claim_user_reward_from_farm(
                    &mut farm, 
                    farmer.get_ref_mut(),  
                    &amount,
                    true,
                );
                self.data_mut().farms.insert(farm_id, &farm);
            }
            self.data_mut().seeds.insert(seed_id, &farm_seed);
            self.data_mut().farmers.insert(sender_id, &farmer);
        }
    }

    pub(crate) fn internal_claim_user_reward_by_farm_id(
        &mut self, 
        sender_id: &AccountId, 
        farm_id: &FarmId) {
        let mut farmer = self.get_farmer(sender_id);

        let (seed_id, _) = parse_farm_id(farm_id);

        if let Some(farm_seed) = self.get_seed_wrapped(&seed_id) {
            let amount = farm_seed.get_ref().amount;
            if let Some(mut farm) = self.data().farms.get(farm_id) {
                claim_user_reward_from_farm(
                    &mut farm, 
                    farmer.get_ref_mut(), 
                    &amount,
                    false,
                );
                self.data_mut().farms.insert(farm_id, &farm);
                self.data_mut().farmers.insert(sender_id, &farmer);
            }
        }
    }


    #[inline]
    pub(crate) fn get_farmer(&self, from: &AccountId) -> VersionedFarmer {
        let orig = self.data().farmers
            .get(from)
            .expect(ERR10_ACC_NOT_REGISTERED);
        if orig.need_upgrade() {
                orig.upgrade()
            } else {
                orig
            }
    }

    #[inline]
    pub(crate) fn get_farmer_default(&self, from: &AccountId) -> VersionedFarmer {
        let orig = self.data().farmers.get(from).unwrap_or(VersionedFarmer::new(from.clone(), 0));
        if orig.need_upgrade() {
            orig.upgrade()
        } else {
            orig
        }
    }

    #[inline]
    pub(crate) fn get_farmer_wrapped(&self, from: &AccountId) -> Option<VersionedFarmer> {
        if let Some(farmer) = self.data().farmers.get(from) {
            if farmer.need_upgrade() {
                Some(farmer.upgrade())
            } else {
                Some(farmer)
            }
        } else {
            None
        }
    }

    /// Returns current balance of given token for given user. 
    /// If there is nothing recorded, returns 0.
    pub(crate) fn internal_get_reward(
        &self,
        sender_id: &AccountId,
        token_id: &AccountId,
    ) -> Balance {
        self.get_farmer_default(sender_id)
            .get_ref().rewards.get(token_id).cloned()
            .unwrap_or_default()
    }
}
