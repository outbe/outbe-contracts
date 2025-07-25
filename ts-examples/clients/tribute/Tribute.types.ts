/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.13.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Denom = {
  native: string;
} | {
  cw20: Addr;
};
export type Addr = string;
export type Decimal = string;
export interface InstantiateMsg {
  burner?: string | null;
  collection_info_extension: TributeCollectionExtension;
  creator?: string | null;
  minter?: string | null;
  name: string;
  symbol: string;
}
export interface TributeCollectionExtension {
  native_token: Denom;
  price_oracle: Addr;
  symbolic_rate: Decimal;
}
export type ExecuteMsg = {
  update_minter_ownership: Action;
} | {
  update_creator_ownership: Action;
} | {
  update_burner_ownership: Action;
} | {
  update_collection_info: {
    collection_info: CollectionInfoMsgForNullable_TributeCollectionExtension;
  };
} | {
  mint: {
    extension: MintExtension;
    owner: string;
    token_id: string;
    token_uri?: string | null;
  };
} | {
  burn: {
    token_id: string;
  };
} | {
  burn_all: {};
};
export type Action = {
  transfer_ownership: {
    expiry?: Expiration | null;
    new_owner: string;
  };
} | "accept_ownership" | "renounce_ownership";
export type Expiration = {
  at_height: number;
} | {
  at_time: Timestamp;
} | {
  never: {};
};
export type Timestamp = Uint64;
export type Uint64 = string;
export type Uint128 = string;
export interface CollectionInfoMsgForNullable_TributeCollectionExtension {
  extension?: TributeCollectionExtension | null;
  name?: string | null;
  symbol?: string | null;
}
export interface MintExtension {
  data: TributeMintData;
}
export interface TributeMintData {
  nominal_qty_minor: Uint128;
  owner: string;
  settlement_amount_minor: Uint128;
  settlement_currency: Denom;
  tribute_id: string;
  tribute_price_minor: Decimal;
  worldwide_day: number;
}
export type QueryMsg = {
  contract_info: {};
} | {
  owner_of: {
    token_id: string;
  };
} | {
  num_tokens: {};
} | {
  get_minter_ownership: {};
} | {
  get_creator_ownership: {};
} | {
  nft_info: {
    token_id: string;
  };
} | {
  tokens: {
    limit?: number | null;
    owner: string;
    query_order?: Order | null;
    start_after?: string | null;
  };
} | {
  all_tokens: {
    limit?: number | null;
    query_order?: Order | null;
    start_after?: string | null;
  };
} | {
  daily_tributes: {
    date: number;
  };
};
export type Order = "ascending" | "descending";
export type MigrateMsg = {
  migrate: {};
};
export interface TokensResponse {
  tokens: string[];
}
export interface ContractInfoResponseForTributeConfig {
  collection_config: TributeConfig;
  collection_info: CollectionInfo;
}
export interface TributeConfig {
  native_token: Denom;
  price_oracle: Addr;
  symbolic_rate: Decimal;
}
export interface CollectionInfo {
  name: string;
  symbol: string;
  updated_at: Timestamp;
}
export interface DailyTributesResponse {
  tributes: FullTributeData[];
}
export interface FullTributeData {
  data: TributeData;
  owner: string;
  token_id: string;
}
export interface TributeData {
  created_at: Timestamp;
  nominal_qty_minor: Uint128;
  settlement_amount_minor: Uint128;
  settlement_currency: Denom;
  symbolic_divisor: Decimal;
  symbolic_load: Uint128;
  tribute_price_minor: Decimal;
  worldwide_day: number;
}
export interface OwnershipForString {
  owner?: string | null;
  pending_expiry?: Expiration | null;
  pending_owner?: string | null;
}
export interface NftInfoResponseForTributeData {
  extension: TributeData;
  owner: Addr;
  token_id: string;
}
export interface NumTokensResponse {
  count: number;
}
export interface OwnerOfResponse {
  owner: string;
}