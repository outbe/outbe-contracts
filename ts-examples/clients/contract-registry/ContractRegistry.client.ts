/**
* This file was automatically generated by @cosmwasm/ts-codegen@1.13.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { InstantiateMsg, ExecuteMsg, Addr, Action, Expiration, Timestamp, Uint64, Deployment, ContractInfo, QueryMsg, AllDeploymentsResponse, GetDeploymentResponse, Binary } from "./ContractRegistry.types";
export interface ContractRegistryReadOnlyInterface {
  contractAddress: string;
  allDeployments: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<AllDeploymentsResponse>;
  getDeployment: ({
    commitId
  }: {
    commitId?: string;
  }) => Promise<GetDeploymentResponse>;
  ownable: () => Promise<Binary>;
}
export class ContractRegistryQueryClient implements ContractRegistryReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.allDeployments = this.allDeployments.bind(this);
    this.getDeployment = this.getDeployment.bind(this);
    this.ownable = this.ownable.bind(this);
  }
  allDeployments = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<AllDeploymentsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_deployments: {
        limit,
        start_after: startAfter
      }
    });
  };
  getDeployment = async ({
    commitId
  }: {
    commitId?: string;
  }): Promise<GetDeploymentResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_deployment: {
        commit_id: commitId
      }
    });
  };
  ownable = async (): Promise<Binary> => {
    return this.client.queryContractSmart(this.contractAddress, {
      ownable: {}
    });
  };
}
export interface ContractRegistryInterface extends ContractRegistryReadOnlyInterface {
  contractAddress: string;
  sender: string;
  publish: ({
    deployment
  }: {
    deployment: Deployment;
  }, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
  ownable: (action: Action, fee_?: number | StdFee | "auto", memo_?: string, funds_?: Coin[]) => Promise<ExecuteResult>;
}
export class ContractRegistryClient extends ContractRegistryQueryClient implements ContractRegistryInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;
  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.publish = this.publish.bind(this);
    this.ownable = this.ownable.bind(this);
  }
  publish = async ({
    deployment
  }: {
    deployment: Deployment;
  }, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      publish: {
        deployment
      }
    }, fee_, memo_, funds_);
  };
  ownable = async (action: Action, fee_: number | StdFee | "auto" = "auto", memo_?: string, funds_?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      ownable: action
    }, fee_, memo_, funds_);
  };
}