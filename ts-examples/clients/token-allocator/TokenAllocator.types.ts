/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.13.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  creator?: string | null;
}
export type ExecuteMsg = string;
export type QueryMsg = {
  get_data: {};
} | {
  get_creator_ownership: {};
} | {
  get_range_data: {
    from_block: Uint64;
    to_block: Uint64;
  };
};
export type Uint64 = string;
export type Expiration = {
  at_height: number;
} | {
  at_time: Timestamp;
} | {
  never: {};
};
export type Timestamp = Uint64;
export interface OwnershipForString {
  owner?: string | null;
  pending_expiry?: Expiration | null;
  pending_owner?: string | null;
}
export interface TokenAllocatorData {
  amount: Uint64;
}