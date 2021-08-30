/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryMsg =
  | ("status" | "get_exchange_settings")
  | {
      get_config: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      get_exchange_address: {
        pair: TokenPairFor_HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      list_idos: {
        pagination: Pagination;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      list_exchanges: {
        pagination: Pagination;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      admin: AdminQueryMsg;
      [k: string]: unknown;
    };
export type TokenPairFor_HumanAddr = [TokenTypeFor_HumanAddr, TokenTypeFor_HumanAddr];
export type TokenTypeFor_HumanAddr =
  | {
      custom_token: {
        contract_addr: HumanAddr;
        token_code_hash: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      native_token: {
        denom: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type HumanAddr = string;
export type AdminQueryMsg = "admin";

export interface Pagination {
  limit: number;
  start: number;
  [k: string]: unknown;
}
