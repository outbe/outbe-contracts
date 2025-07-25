/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.13.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Uint128, InstantiateMsg, Vector, ExecuteMsg, QueryMsg, Expiration, Timestamp, Uint64, OwnershipForString, AllVectorsResponse } from "./Vector.types";
export interface VectorReadOnlyInterface {
  contractAddress: string;
  vectors: () => Promise<AllVectorsResponse>;
  getCreatorOwnership: () => Promise<OwnershipForString>;
}
export class VectorQueryClient implements VectorReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.vectors = this.vectors.bind(this);
    this.getCreatorOwnership = this.getCreatorOwnership.bind(this);
  }
  vectors = async (): Promise<AllVectorsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      vectors: {}
    });
  };
  getCreatorOwnership = async (): Promise<OwnershipForString> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_creator_ownership: {}
    });
  };
}