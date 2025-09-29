use crate::traits::{Cw721CollectionConfig, Cw721State};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, StdError, StdResult, Storage, Timestamp};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

/// Creator owns this contract and can update collection info!
/// !!! Important note here: !!!
/// - creator is stored using cw-ownable's OWNERSHIP singleton, so it is not stored here
/// - in release v0.18.0 it was used for minter (which is confusing), but now it is used for creator
pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);
/// - minter is stored in the contract storage using cw_ownable::OwnershipStore (same as for OWNERSHIP but with different key)
pub const MINTER: OwnershipStore = OwnershipStore::new("collection_minter");

pub const BURNER: OwnershipStore = OwnershipStore::new("collection_burner");

#[cw_serde]
pub struct CollectionInfo {
    pub name: String,
    pub symbol: String,
    pub updated_at: Timestamp,
}

pub struct Cw721Config<'a, TNftExtension, TCollectionConfig>
where
    TNftExtension: Cw721State,
    TCollectionConfig: Cw721CollectionConfig,
{
    pub collection_info: Item<CollectionInfo>,
    pub collection_config: Item<TCollectionConfig>,
    pub token_count: Item<u64>,
    pub nft_info: IndexedMap<&'a str, NftInfo<TNftExtension>, TokenIndexes<'a, TNftExtension>>,
}

impl<TNftExtension, TCollectionConfig> Default
    for Cw721Config<'static, TNftExtension, TCollectionConfig>
where
    TNftExtension: Cw721State,
    TCollectionConfig: Cw721CollectionConfig,
{
    fn default() -> Self {
        Self::new(
            "cw721_collection_info",
            "cw721_collection_config",
            "num_tokens",
            "tokens",
            "tokens__owner",
        )
    }
}

impl<TNftExtension, TCollectionConfig> Cw721Config<'_, TNftExtension, TCollectionConfig>
where
    TNftExtension: Cw721State,
    TCollectionConfig: Cw721CollectionConfig,
{
    fn new(
        collection_info_key: &'static str,
        collection_config_key: &'static str,
        token_count_key: &'static str,
        nft_info_key: &'static str,
        nft_info_owner_key: &'static str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, nft_info_key, nft_info_owner_key),
        };
        Self {
            collection_info: Item::new(collection_info_key),
            collection_config: Item::new(collection_config_key),
            token_count: Item::new(token_count_key),
            nft_info: IndexedMap::new(nft_info_key, indexes),
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
    pub fn clean_tokens(
        &self,
        storage: &mut dyn Storage,
        batch_size: Option<usize>,
    ) -> StdResult<()> {
        // NB clean indexes as well
        let batch: usize = batch_size.unwrap_or(usize::MAX);

        let paths = self
            .nft_info
            .prefix_range(storage, None, None, Order::Ascending)
            .filter_map(|item| item.ok())
            .take(batch)
            .collect::<Vec<_>>();

        let paths_len = paths.len() as u64;
        for (pk, value) in paths {
            self.nft_info
                .idx
                .owner
                .remove(storage, pk.as_bytes(), &value)?;
            self.nft_info.remove(storage, &pk)?;
        }
        self.token_count.update(storage, |val| {
            if val > paths_len {
                Ok::<u64, StdError>(val - paths_len)
            } else {
                Ok(0)
            }
        })?;

        Ok(())
    }
}

pub fn token_owner_idx<TNftExtension>(_pk: &[u8], d: &NftInfo<TNftExtension>) -> Addr {
    d.owner.clone()
}

#[cw_serde]
pub struct NftInfo<TNftExtension> {
    /// The owner of the newly minted NFT
    pub owner: Addr,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: TNftExtension,
}

pub struct TokenIndexes<'a, TNftExtension>
where
    TNftExtension: Cw721State,
{
    pub owner: MultiIndex<'a, Addr, NftInfo<TNftExtension>, String>,
}

impl<TNftExtension> IndexList<NftInfo<TNftExtension>> for TokenIndexes<'_, TNftExtension>
where
    TNftExtension: Cw721State,
{
    fn get_indexes(
        &'_ self,
    ) -> Box<dyn Iterator<Item = &'_ dyn Index<NftInfo<TNftExtension>>> + '_> {
        let v: Vec<&dyn Index<NftInfo<TNftExtension>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}
