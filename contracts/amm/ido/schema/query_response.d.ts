/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryResponse =
  | {
      eligibility: {
        can_participate: boolean;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      sale_info: {
        /**
         * Sale end time.
         */
        end?: number | null;
        /**
         * The token that is used to buy the sold SNIP20.
         */
        input_token: TokenType;
        /**
         * The total amount that each participant is allowed to buy.
         */
        max_allocation: Uint128;
        /**
         * The maximum number of participants allowed.
         */
        max_seats: number;
        /**
         * The minimum amount that each participant is allowed to buy.
         */
        min_allocation: Uint128;
        /**
         * The conversion rate at which the token is sold.
         */
        rate: Uint128;
        /**
         * The token that is being sold.
         */
        sold_token: ContractLink;
        /**
         * Sale start time.
         */
        start?: number | null;
        /**
         * Number of participants currently.
         */
        taken_seats: number;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      status: {
        available_for_sale: Uint128;
        is_active: boolean;
        sold_in_pre_lock: Uint128;
        total_allocation: Uint128;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      balance: {
        pre_lock_amount: Uint128;
        total_bought: Uint128;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      token_info: {
        decimals: number;
        name: string;
        symbol: string;
        total_supply?: Uint128 | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
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
export type HumanAddr = string;
export type Uint128 = string;

/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLink {
  address: HumanAddr;
  code_hash: string;
}
