use crate::state::CollectionInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Cw721InstantiateMsg<TCollectionExtensionMsg> {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    /// Optional extension of the collection metadata
    pub collection_info_extension: TCollectionExtensionMsg,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: Option<String>,

    /// Sets the creator of collection. The creator is the only one eligible to update `CollectionInfo`.
    pub creator: Option<String>,
}

/// This is a wrapper around CollectionInfo that includes the extension, contract info, and number of tokens (supply).
#[cw_serde]
pub struct ContractInfoResponse<TCollectionExtensionMsg> {
    pub collection_info: CollectionInfo,
    pub collection_config: TCollectionExtensionMsg,
}

#[cw_serde]
pub struct OwnerOfResponse {
    /// Owner of the token
    pub owner: String,
}

#[cw_serde]
pub struct NumTokensResponse {
    pub count: u64,
}

#[cw_serde]
pub struct NftInfoResponse<TNftExtension> {
    /// You can add any custom metadata here when you extend cw721-base
    pub extension: TNftExtension,
    pub owner: Addr,
}

#[cw_serde]
pub struct TokensResponse {
    /// Contains all token_ids in lexicographical ordering
    /// If there are more than `limit`, use `start_after` in future queries
    /// to achieve pagination.
    pub tokens: Vec<String>,
}
