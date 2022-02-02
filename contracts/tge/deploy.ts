import {
  MigrationContext, printContracts, Deployment, Chain, Agent,
  bold, Console, randomHex, timestamp
} from '@hackbg/fadroma'

const console = Console('@sienna/amm/upgrade')

import type { ScheduleFor_HumanAddr } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract
} from '@sienna/api'

import * as settings from '@sienna/settings'

export async function deployTGE ({
  chain, agent, deployment, prefix,
  schedule = settings.schedule
}: MigrationContext & {
  /** Input: The schedule for the new MGMT.
    * Defaults to production schedule. */
  schedule?: typeof settings.schedule
}): Promise<{
  /** Output: The newly created deployment. */
  deployment: Deployment
  /** Output: The identifier of the deployment on- and off-chain. */
  prefix:     string
  /** Output: The deployed SIENNA SNIP20 token contract. */
  SIENNA:     SiennaSNIP20Contract
  /** Output: The deployed MGMT contract. */
  MGMT:       MGMTContract
  /** Output: The deployed RPT contract. */
  RPT:        RPTContract
}> {

  const [SIENNA, MGMT, RPT] = await chain.buildAndUpload(agent, [
    new SiennaSNIP20Contract(),
    new MGMTContract(),
    new RPTContract()
  ])

  await deployment.init(agent, SIENNA, {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  })

  const admin = agent.address

  if (chain.isTestnet || chain.isLocalnet) {
    await SIENNA.tx(agent).setMinters([admin])
    await SIENNA.tx(agent).mint("5000000000000000000000", admin)
  }

  const RPTAccount = getRPTAccount(schedule)
  RPTAccount.address = admin // mutate schedule
  const portion = RPTAccount.portion_size

  await deployment.init(agent, MGMT, {
    admin: admin,
    token: [SIENNA.address, SIENNA.codeHash],
    schedule
  })

  await MGMT.tx().acquire(SIENNA)

  await deployment.init(agent, RPT, {
    token:   [SIENNA.address, SIENNA.codeHash],
    mgmt:    [MGMT.address, MGMT.codeHash],
    portion: RPTAccount.portion_size,
    config:  [[admin, RPTAccount.portion_size]]
  })

  console.log()
  console.info(bold('Deployed TGE contracts:'))
  printContracts([SIENNA, MGMT, RPT])

  console.info(bold('Setting TGE schedule'))
  RPTAccount.address = RPT.address
  await MGMT.tx().configure(schedule)

  console.info(bold('Launching the TGE'))
  await MGMT.tx().launch()

  console.info(bold('Vesting RPT'))
  await RPT.tx().vest()

  return {
    deployment,
    prefix,
    SIENNA,
    MGMT,
    RPT
  }

  /// ### Get the RPT account from the schedule
  /// This is a special entry in MGMT's schedule that must be made to point to
  /// the RPT contract's address - but that's only possible after deploying
  /// the RPT contract. To prevent the circular dependency, the RPT account
  /// starts as pointing to the admin's address.
  function getRPTAccount (schedule: ScheduleFor_HumanAddr) {
    return schedule.pools
      .filter((x:any)=>x.name==='MintingPool')[0].accounts
      .filter((x:any)=>x.name==='RPT')[0] }
}
