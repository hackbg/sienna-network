#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, basename, extname, dirname } from 'path'
import { env, argv, stdout, stderr, exit } from 'process'
import { fileURLToPath } from 'url'
import open from 'open'
import yargs from 'yargs'
import { SecretNetwork } from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import Localnet from '@fadroma/scrt-ops/localnet.js'
import { scheduleFromSpreadsheet } from '@sienna/schedule'
import { args, combine } from './args.js'
import { genConfig, genCoverage, genSchema, genDocs } from './gen.js'
import { abs, stateBase } from './root.js'
import { clear, cargo, run, runTests, runDemo } from './run.js'
import TGEContracts from './TGEContracts.js'
import RewardsContracts from './RewardsContracts.js'

export default function main () {
  let cmd = yargs(process.argv.slice(2))
    .wrap(yargs().terminalWidth())
    .demandCommand(1, '')

  // validation:
  cmd = cmd
    .command('test',
      '⚗️  Run test suites for all the individual components.',
      runTests)
    .command('ensure-wallets',
      '⚗️  Ensure there are testnet wallets for the demo.',
      ensureWallets)
    .command('demo [--testnet]',
      '⚗️  Run integration test/demo.',
      args.IsTestnet, runDemo)
    .command('clean-localnet',
      '♻️  Try to terminate a loose localnet container and remove its state files',
      () => new Localnet().terminate())

  // artifacts:
  cmd = cmd
    .command('schema',
      `🤙 Regenerate JSON schema for each contract's API.`,
      genSchema)
    .command('docs [crate]',
      '📖 Build the documentation and open it in a browser.',
      args.Crate, genDocs)
    .command('coverage',
      '⚗️  Generate test covera@asparuhge and open it in a browser.',
      genCoverage)
    .command('config [<spreadsheet>]',
      '📅 Convert a spreadsheet into a JSON schedule',
      args.Spreadsheet, genConfig)

  cmd = new TGEContracts().commands(cmd)

  cmd = new RewardsContracts().commands(cmd)

  return cmd.argv
}

try {
  main()
} catch (e) {
  console.error(e)
  const ISSUES = `https://github.com/hackbg/sienna-secret-token/issues`
  console.info(`👹 That was a bug. Report it at ${ISSUES}`)
}
