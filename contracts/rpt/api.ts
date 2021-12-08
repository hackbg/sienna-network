import type { Agent } from '@fadroma/ops'
import type { LinearMapFor_HumanAddrAnd_Uint128, Uint128 } from './rpt/init'
import { ScrtContract, loadSchemas } from "@fadroma/scrt"
import { abs } from '../ops/index'

export class RPTContract extends ScrtContract {

  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./rpt/init.json",
    queryMsg:    "./rpt/query.json",
    queryAnswer: "./rpt/response.json",
    handleMsg:   "./rpt/handle.json"
  })

  code = { ...this.code, workspace: abs(), crate: 'sienna-rpt' }

  init = { ...this.init, label: 'SiennaRPT', msg: {} }

  constructor (options: {
    prefix?:  string
    admin?:   Agent
    config?:  LinearMapFor_HumanAddrAnd_Uint128
    portion?: Uint128
    SIENNA?:  SiennaSNIP20
    MGMT?:    MGMTContract
  } = {}) {
    super({
      prefix: options?.prefix,
      agent:  options?.admin,
      schema: RPTContract.schema
    })
    Object.assign(this.init.msg, {
      token:   options?.SIENNA?.linkPair,
      mgmt:    options?.MGMT?.linkPair,
      portion: options.portion,
      config:  [[options.admin?.address, options.portion]]
    })
    Object.defineProperties(this.init.msg, {
      token: { enumerable: true, get () { return options?.SIENNA?.linkPair } },
      mgmt:  { enumerable: true, get () { return options?.MGMT?.linkPair   } }
    })
  }

  /** query contract status */
  get status() { return this.q.status().then(({status})=>status) }

  /** set the splitt proportions */
  configure = (config = []) => this.tx.configure({ config })

  /** claim portions from mgmt and distribute them to recipients */
  vest = () => this.tx.vest()

  /** set the admin */
  setOwner = (new_admin) => this.tx.set_owner({ new_admin })

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new RPTContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

}

