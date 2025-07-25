/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.13.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export interface InstantiateMsg {
  admin?: string | null;
  mint?: MinterResponse | null;
}
export interface MinterResponse {
  cap?: Uint128 | null;
  minter: string;
}
export type ExecuteMsg = {
  burn: {
    amount: Uint128;
  };
} | {
  mint: {
    amount: Uint128;
    recipient: string;
  };
} | {
  update_minter: {
    new_minter?: string | null;
  };
} | {
  update_admin: {
    new_admin: string;
  };
};
export type QueryMsg = {
  balance: {
    address: string;
  };
} | {
  token_info: {};
} | {
  minter: {};
} | {
  all_accounts: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  check_ticket: {
    ticket: string;
  };
} | {
  admin: {};
};
export type String = string;
export interface AllAccountsResponse {
  accounts: string[];
}
export interface BalanceResponse {
  balance: Uint128;
}
export interface CheckTicketResponse {
  exists: boolean;
}
export interface TokenInfoResponse {
  decimals: number;
  name: string;
  symbol: string;
  total_supply: Uint128;
}