import type { IAgent } from "@fadroma/scrt";
import { ScrtContract_1_2, ContractAPIOptions } from "@fadroma/scrt";
import { randomHex } from "@hackbg/tools";

import { b64encode } from "@waiting/base64";
import { EnigmaUtils } from "secretjs/src/index.ts";

import { AMMContract        } from "@sienna/exchange";
import { AMMSNIP20Contract  } from "@sienna/amm-snip20";
import { LPTokenContract    } from "@sienna/lp-token";
import { IDOContract        } from "@sienna/ido";
import { LaunchpadContract  } from "@sienna/launchpad";

import { workspace } from "@sienna/settings";

import {
  ExchangeSettings,
  ContractInstantiationInfo
} from './schema/init_msg.d';
import { TokenType } from './schema/handle_msg.d';
import { QueryResponse, Exchange } from './schema/query_response.d'

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

// Okay, so here's how a deployer class works.
export class FactoryContract extends ScrtContract_1_2 {

  // Schema: used for generating the .q and .tx methods,
  // supposedly validating inputs and outputs.
  static schema = ScrtContract_1_2.loadSchemas(import.meta.url, {
    initMsg:     "./schema/init_msg.json",
    queryMsg:    "./schema/query_msg.json",
    queryAnswer: "./schema/query_response.json",
    handleMsg:   "./schema/handle_msg.json",
  });

  static attach = (
    address:  string,
    codeHash: string,
    agent:    IAgent
  ) => {
    const instance = new FactoryContract({ admin: agent })
    instance.init.agent    = agent
    instance.init.address  = address
    instance.blob.codeHash = codeHash
    return instance
  }

  constructor({
    codeId,
    admin,
    schema = FactoryContract.schema,
    prefix,
    exchange_settings,
    contracts
  }: ContractAPIOptions & {

    admin?: IAgent

    /* AMM config from project settings.
     * First auto-generated type definition
     * finally put to use in the codebase! Hooray */
    exchange_settings?:  ExchangeSettings

    /* Contract contracts (id + codehash)
     * for each contract known by the factory */
    contracts?: FactoryInventory

  } = {}) {

    super({
      // for building
      workspace, crate: 'factory',
      // for uploading
      codeId, label: 'SiennaAMMFactory',
      // for transacting
      schema, prefix, agent: admin
    })

    Object.assign(this.init.msg, {
      admin: admin?.address,
      prng_seed: randomHex(36),
      exchange_settings: exchange_settings || {
        swap_fee: { nom: 28, denom: 1000 },
        sienna_fee: { nom: 2, denom: 10000 },
        sienna_burner: null,
      },
      ...(contracts||{}),
    })

  }

  get contracts (): Promise<FactoryInventory> {
    if (this.address) {
      // If this contract has an address query this from the contract state
      return (this.q.get_config() as Promise<QueryResponse>).then(({
        snip20_contract,
        pair_contract,
        lp_token_contract,
        ido_contract,
        launchpad_contract,
        router_contract
      })=>({// type kludge!
        snip20_contract:    snip20_contract    as ContractInstantiationInfo,
        pair_contract:      pair_contract      as ContractInstantiationInfo,
        lp_token_contract:  lp_token_contract  as ContractInstantiationInfo,
        ido_contract:       ido_contract       as ContractInstantiationInfo,
        launchpad_contract: launchpad_contract as ContractInstantiationInfo,
        router_contract:    router_contract    as ContractInstantiationInfo
      }))
    } else {
      // If it's not deployed yet, return the value from the config
      return Promise.resolve({
        snip20_contract:    this.init.msg.snip20_contract,
        pair_contract:      this.init.msg.pair_contract,
        lp_token_contract:  this.init.msg.lp_token_contract,
        ido_contract:       this.init.msg.ido_contract,
        launchpad_contract: this.init.msg.launchpad_contract,
        router_contract:    this.init.msg.router_contract
      })
    }
  }

  async listExchanges (): Promise<Exchange[]> {
    const result: Exchange[] = []
    const limit = 30

    let start = 0
    while(true) {
      const response: QueryResponse = await this.q.listExchanges({ pagination: { start, limit } }) as QueryResponse
      const list:     Exchange[]    = (response.list_exchanges as { exchanges: Exchange[] }).exchanges
      if (list.length == 0) {
        break
      }
      result.push.apply(result, list)
      start += limit
    }

    return result
  }

  async getExchange (token_0: TokenType, token_1: TokenType, agent = this.instantiator) {

    const pair = { token_0, token_1 }

    const {get_exchange_address:{address:exchange_address}} =
      await agent.query(this.link, "get_exchange_address", { pair })

    const exchange = AMMContract.attach(exchange_address, undefined, agent)

    const {pair_info:{liquidity_token:lp_token}} = await exchange.pairInfo()

    return { exchange: exchange.link, lp_token, token_0, token_1 }

  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract */
  async createExchange (token_0: TokenType, token_1: TokenType, agent = this.instantiator) {
    const pair = { token_0, token_1 };
    const entropy = b64encode(EnigmaUtils.GenerateNewSeed().toString());
    await agent.execute(this.link, "create_exchange", { pair, entropy });
    return await this.getExchange(token_0, token_1, agent);
  }

  /** Create an instance of the launchpad contract. */
  createLaunchpad (tokens: object[], agent = this.instantiator) {
    return this.tx.create_launchpad({ tokens, }, agent)
  }

}
