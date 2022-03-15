//! FarmSeed stores information per seed about 
//! staked seed amount and farms under it.

use std::collections::HashSet;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Balance};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{U128};
use crate::errors::*;
use crate::farm::FarmId;
use crate::utils::parse_seed_id;
use std::collections::HashMap;
use near_sdk::collections::LookupMap;
use crate::{Contract, StorageKeys};


/// For MFT, SeedId composes of token_contract_id 
/// and token's inner_id in that contract. 
/// For FT, SeedId is the token_contract_id.
pub(crate) type SeedId = String;

pub(crate) type NFTTokenId = String; //paras-comic-dev.testnet@6

pub(crate) type NftBalance = HashMap<NFTTokenId, U128>; //paras-comic-dev.testnet@6

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum SeedType {
    FT,
    MFT,
    NFT
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmSeedMetadata {
    pub title: Option<String>,
    pub media: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct FarmSeedV1 {
    /// The Farming Token this FarmSeed represented for
    pub seed_id: SeedId,
    /// The seed is a FT or MFT or NFT
    pub seed_type: SeedType,
    /// all farms that accepted this seed
    /// FarmId = {seed_id}#{next_index}
    pub farms: HashSet<FarmId>,
    pub next_index: u32,
    /// total (staked) balance of this seed (Farming Token)
    pub amount: Balance,
    pub min_deposit: Balance,
    pub nft_balance: Option<HashMap<NFTTokenId, U128>>,
    pub metadata: Option<FarmSeedMetadata>
}

#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct FarmSeed {
    /// The Farming Token this FarmSeed represented for
    pub seed_id: SeedId,
    /// The seed is a FT or MFT or NFT
    pub seed_type: SeedType,
    /// all farms that accepted this seed
    /// FarmId = {seed_id}#{next_index}
    pub farms: HashSet<FarmId>,
    pub next_index: u32,
    /// total (staked) balance of this seed (Farming Token)
    pub amount: Balance,
    pub min_deposit: Balance,
    pub metadata: Option<FarmSeedMetadata>
}

impl FarmSeed {
    pub fn new(
        seed_id: &SeedId,
        min_deposit: Balance,
        is_nft_balance: bool,
        metadata: Option<FarmSeedMetadata>
    ) -> Self {
        let (token_id, token_index) = parse_seed_id(seed_id);
        let seed_type: SeedType;
        if is_nft_balance {
            seed_type = SeedType::NFT;
        } else if token_id == token_index {
            seed_type = SeedType::FT; // If NFT, then SeedId will indicate the balance equivalent instead of adding seed with FT
        } else {
            seed_type = SeedType::MFT;
        }
        Self {
            seed_id: seed_id.clone(),
            seed_type,
            farms: HashSet::new(),
            next_index: 0,
            amount: 0,
            min_deposit,
            metadata
        }
    }

    pub fn add_amount(&mut self, amount: Balance) {
        self.amount += amount;
    }

    /// return seed amount remains.
    pub fn sub_amount(&mut self, amount: Balance) -> Balance {
        assert!(self.amount >= amount, "{}", ERR500);
        self.amount -= amount;
        self.amount
    }

}

/// Versioned FarmSeed, used for lazy upgrade.
/// Which means this structure would upgrade automatically when used.
/// To achieve that, each time the new version comes in, 
/// each function of this enum should be carefully re-code!
#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedFarmSeed {
    V101(FarmSeedV1),
    V102(FarmSeed),
}

impl VersionedFarmSeed {

    pub fn new(
        seed_id: &SeedId,
        min_deposit: Balance,
        is_nft_balance: bool,
        metadata: Option<FarmSeedMetadata>,
    ) -> Self {
        VersionedFarmSeed::V102(FarmSeed::new(seed_id, min_deposit, is_nft_balance, metadata))
    }

    /// Upgrades from other versions to the currently used version.
    pub fn upgrade(self, contract: &mut Contract) -> Self {
        match self {
            VersionedFarmSeed::V102(farm_seed) => VersionedFarmSeed::V102(farm_seed),
            VersionedFarmSeed::V101(farm_seed) => {
                if let Some(nft_balance) = farm_seed.nft_balance {
                    contract.data_mut().nft_balance_seeds.insert(&farm_seed.seed_id, &nft_balance);
                }
                return VersionedFarmSeed::V102(FarmSeed {
                    seed_id: farm_seed.seed_id,
                    seed_type: farm_seed.seed_type,
                    farms: farm_seed.farms,
                    next_index: farm_seed.next_index,
                    amount: farm_seed.amount,
                    min_deposit: farm_seed.min_deposit,
                    metadata: farm_seed.metadata,
                })
            }
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn need_upgrade(&self) -> bool {
        match self {
            VersionedFarmSeed::V102(_) => false,
            _ => true,
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn get_ref(&self) -> &FarmSeed {
        match self {
            VersionedFarmSeed::V102(farm_seed) => farm_seed,
            _ => unimplemented!(),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    pub fn get_ref_mut(&mut self) -> &mut FarmSeed {
        match self {
            VersionedFarmSeed::V102(farm_seed) => farm_seed,
            _ => unimplemented!(),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SeedInfo {
    pub seed_id: SeedId,
    pub seed_type: String,
    pub farms: Vec<FarmId>,
    pub next_index: u32,
    pub amount: U128,
    pub min_deposit: U128,
    pub nft_balance: Option<NftBalance>,
    pub title: Option<String>,
    pub media: Option<String>
}

impl From<&FarmSeed> for SeedInfo {
    fn from(fs: &FarmSeed) -> Self {
        let seed_type = match fs.seed_type {
            SeedType::FT => "FT".to_string(),
            SeedType::NFT => "NFT".to_string(),
            SeedType::MFT => "MFT".to_string(),
        };
        if let Some(seed_metadata) = fs.metadata.clone() {
            Self {
                seed_id: fs.seed_id.clone(),
                seed_type,
                next_index: fs.next_index,
                amount: fs.amount.into(),
                min_deposit: fs.min_deposit.into(),
                farms: fs.farms.iter().map(|key| key.clone()).collect(),
                title: Some(seed_metadata.title.unwrap_or("".to_string())),
                media: Some(seed_metadata.media.unwrap_or("".to_string())),
                nft_balance: None,
            }
        } else {
            Self {
                seed_id: fs.seed_id.clone(),
                seed_type,
                next_index: fs.next_index,
                amount: fs.amount.into(),
                min_deposit: fs.min_deposit.into(),
                farms: fs.farms.iter().map(|key| key.clone()).collect(),
                title: Some("".to_string()),
                media: Some("".to_string()),
                nft_balance: None
            }
        }
    }
}
