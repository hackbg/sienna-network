import {
  MigrationContext, Source, Builder, Uploader, Template,
  buildAndUploadMany
} from '@hackbg/fadroma'

import * as API from '@sienna/api'

import { versions, contracts, sources } from '../Build'
import { linkTuple } from '../misc'

export type Address = string
export type Binary  = string
export type GitRef  = string

export interface LaunchpadDeployOptions {
  /** Address of the admin. */
  admin:     Address
  /** From which Git commit to build. */
  ref?:      GitRef
  /** Source handles of contracts from selected Git ref */
  src?:      Source[]
  builder:   Builder
  uploader:  Uploader
  templates: Template[]
}

export interface LaunchpadDeployResult {
  /** The deployed LPD contract. */
  LPD: API.LaunchpadClient
}

export async function deployLaunchpad (
  context: MigrationContext & LaunchpadDeployOptions
): Promise<LaunchpadDeployResult> {

  const {
    ref = versions.HEAD,
    src = sources(ref, contracts.Launchpad),
    builder,
    uploader,
    templates = await buildAndUploadMany(builder, uploader, src),
    deployment,
    prefix,
    agent,
    admin = agent.address,
  } = context

  // 1. Build and upload LPD contracts:
  const [launchpadTemplate, idoTemplate] = templates

  // 2. Instantiate the launchpad contract 
  let prng_seed: Binary = "";
  let entropy: Binary = "";

  const lpdInstance = await deployment.init(agent, launchpadTemplate, 'LPD', {
    admin: admin,
    tokens: [],
    prng_seed: prng_seed,
    entropy: entropy
  })

  const lpdLink = linkTuple(lpdInstance)

  // 3. Return interfaces to the contracts.
  //    This will add them to the context for
  //    subsequent steps. (Retrieves them through
  //    the Deployment to make sure the receipts
  //    were saved.)
  const client = (Class, name) => new Class({...deployment.get(name), agent})

  return {
    LPD: client(API.LaunchpadClient, 'LPD')
  }

}
