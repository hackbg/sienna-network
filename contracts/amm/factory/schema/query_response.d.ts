/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryResponse =
  | {
      get_exchange_address: {
        address: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      get_launchpad_address: {
        address: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      list_idos: {
        idos: HumanAddr[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      list_exchanges: {
        exchanges: Exchange[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      get_exchange_settings: {
        settings: ExchangeSettings;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      config: {
        exchange_settings: ExchangeSettings;
        ido_contract: ContractInstantiationInfo;
        launchpad_contract: ContractInstantiationInfo;
        lp_token_contract: ContractInstantiationInfo;
        pair_contract: ContractInstantiationInfo;
        snip20_contract: ContractInstantiationInfo;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type HumanAddr = string;
export type TokenPair = [TokenType, TokenType];
export type TokenType =
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

/**
 * Represents the address of an exchange and the pair that it manages
 */
export interface Exchange {
  /**
   * The contract that manages the exchange.
   */
  contract: ContractLink;
  /**
   * The pair that the contract manages.
   */
  pair: TokenPair;
  [k: string]: unknown;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLink {
  address: HumanAddr;
  code_hash: string;
}
export interface ExchangeSettings {
  sienna_burner?: HumanAddr | null;
  sienna_fee: Fee;
  swap_fee: Fee;
  [k: string]: unknown;
}
export interface Fee {
  denom: number;
  nom: number;
  [k: string]: unknown;
}
/**
 * Info needed to instantiate a contract.
 */
export interface ContractInstantiationInfo {
  code_hash: string;
  id: number;
}
