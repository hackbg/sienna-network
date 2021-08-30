/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type HumanAddr = string;
export type Uint128 = string;
export type ViewingKey = string;

export interface Init {
  admin?: HumanAddr | null;
  cooldown?: number | null;
  lp_token?: ContractInstanceFor_HumanAddr | null;
  ratio?: [Uint128, Uint128] | null;
  reward_token: ContractInstanceFor_HumanAddr;
  threshold?: number | null;
  viewing_key: ViewingKey;
  [k: string]: unknown;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractInstanceFor_HumanAddr {
  address: HumanAddr;
  code_hash: string;
  [k: string]: unknown;
}
