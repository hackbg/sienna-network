import {
  MigrationContext,
  IChain, IAgent,
  bold, randomHex, timestamp,
  writeFileSync,
  Console
} from '@hackbg/fadroma'

const console = Console('@sienna/amm/deploySwap')

import type { SNIP20Contract } from '@fadroma/snip20'

import getSettings, { workspace, SIENNA_DECIMALS, ONE_SIENNA } from '@sienna/settings'

import {
  FactoryContract,

  AMMContract,
  AMMSNIP20Contract,
  IDOContract,
  LPTokenContract,
  LaunchpadContract,

  RPTContract,
  RewardsContract,
  SiennaSNIP20Contract,
} from '@sienna/api'

import { deployPlaceholderTokens } from './deployPlaceholderTokens'
import { getSwapTokens } from './getSwapTokens'
import { deployRewardPool } from './deployRewardPool'
import { deployLiquidityPool } from './deployLiquidityPool'

export async function deploySwap (inputs: MigrationContext & {
  SIENNA?:  SiennaSNIP20Contract
  RPT?:     RPTContract
  settings: { amm: { exchange_settings: any } }
}) {

  const {
    chain,
    admin,
    deployment,
    prefix,
    run,

    // expected contracts
    // get them from the deployment
    // or from the previous stage (deployVesting.ts)
    SIENNA = new SiennaSNIP20Contract().from(deployment),
    RPT    = new RPTContract().from(deployment),

    // hardcoded initial settings for this chain
    settings = getSettings(chain.chainId)

  } = inputs

  const { chainId, isLocalnet, isMainnet } = chain

  const options = { workspace, prefix, admin }

  const [
    // only this one will be deployed
    FACTORY,
    // however all of them must be built, so that
    // the factory can be given their code ids/hashes
    EXCHANGE,
    AMMTOKEN,
    LPTOKEN,
    IDO,
    LAUNCHPAD
  ] = await chain.buildAndUpload([
    new FactoryContract({   ...options }),
    new AMMContract({       ...options }),
    new AMMSNIP20Contract({ ...options }),
    new LPTokenContract({   ...options }),
    new IDOContract({       ...options }),
    new LaunchpadContract({ ...options }),
  ])

  const template = contract => ({
    id:        contract.codeId,
    code_hash: contract.codeHash,
  })

  // configure factory: set supported contracts
  FACTORY.setContracts({
    snip20_contract:    template(AMMTOKEN),
    pair_contract:      template(EXCHANGE),
    lp_token_contract:  template(LPTOKEN),
    ido_contract:       template(IDO),
    launchpad_contract: template(LAUNCHPAD),
  })

  // configure factory: set fees etc
  const { amm: { exchange_settings } } = settings
  FACTORY.initMsg.exchange_settings = exchange_settings

  // deploy the factory
  await FACTORY.instantiateOrExisting(deployment.contracts['SiennaAMMFactory'])

  // obtain a list of token addr/hash pairs for creating liquidity pools
  const tokens: Record<string, SNIP20Contract> = { SIENNA }
  if (isLocalnet) {
    // On localnet, placeholder tokens need to be deployed.
    console.info(`Running on ${bold('localnet')}, deploying placeholder tokens...`)
    Object.assign(tokens, await run(deployPlaceholderTokens))
  } else {
    // On testnet and mainnet, interoperate with preexisting token contracts.
    console.info(`Not running on localnet, using tokens from config:`)
    Object.assign(tokens, getSwapTokens(settings.swapTokens))
    console.debug('Tokens', tokens)
  }

  // Deploy pools and add them to the RPT configuration.

  // 1. Stake SIENNA to earn SIENNA
  const singleSidedStaking = await run(deployRewardPool, {
    lpToken:     SIENNA,
    rewardToken: SIENNA,
  })

  // 2. Add that to the RPT config
  const rptConfig: [string, string][] = [
    [
      singleSidedStaking.address,
      String(BigInt(settings.rewardPairs.SIENNA) * ONE_SIENNA)
    ]
  ]

  // 3. If there are any initial swap pairs defined
  const swapPairs = settings.swapPairs
  if (swapPairs.length > 0) {

    const existingExchanges = await FACTORY.listExchanges()
    const rewardPairs = settings.rewardPairs
    const liquidityPoolOptions = {
      admin,
      FACTORY,
      existingExchanges,
      tokens,
      deployment
    }

    for (const name of swapPairs) {

      // 4. Instantiate each one in the factory,
      //    keeping the handle to the LP token
      const {lp_token} = await run(deployLiquidityPool, {
        ...liquidityPoolOptions,
        name
      })

      // 5. If this swap pair has an assigned reward pool in the config
      if (rewardPairs && rewardPairs[name]) {

        console.info(`Deploying rewards for ${name}...`)

        const reward = String(BigInt(rewardPairs[name]) * ONE_SIENNA)

        // 6. Stake LP to earn sienna. 
        const pool = await run(deployRewardPool, {
          prefix,
          rewardToken: SIENNA,
          lpToken: new LPTokenContract({
            address:  lp_token.address,
            codeHash: lp_token.code_hash,
            admin
          }),
        })

        // 7. Add that to the RPT config
        rptConfig.push([pool.address, reward])

      }
    }

  }

  if (isMainnet) {
    const rptConfigPath = deployment.resolve(`RPTConfig.json`)
    writeFileSync(rptConfigPath, JSON.stringify({config: rptConfig}, null, 2), 'utf8')
    console.info(
      `\n\nWrote ${bold(rptConfigPath)}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
  } else {
    console.info(`Configuring RPT to fund ${bold(String(rptConfig.length))} reward pools:`)
    for (const [address, amount] of rptConfig) {
      console.info(`- ${address} ${amount}`)
    }
    await RPT.tx(admin).configure(rptConfig)
  }

}
