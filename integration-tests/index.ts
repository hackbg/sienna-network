import { 
  ContractInfo, TokenPair, Address, TokenPairAmount,
  ViewingKey, Uint128, TokenTypeAmount, Pagination
} from './amm-lib/types.js'
import { FactoryContract, ExchangeContract, Snip20Contract } from './amm-lib/contract.js'
import { 
  execute_test, execute_test_expect, assert_objects_equal, assert,
  assert_equal, assert_not_equal, extract_log_value, print_object
} from './test_helpers.js'
import { setup, build_client } from './setup.js'
import { NullJsonFileWriter } from './utils/json_file_writer.js'
import { SigningCosmWasmClient, Account } from 'secretjs'
import { Sha256, Random } from "@iov/crypto"
import { Buffer } from 'buffer'

import.meta.url

interface LocalAccount {
  name: string,
  type: string,
  address: string,
  pubkey: string,
  mnemonic: string
}

const APIURL = 'http://localhost:1337'

const ACC: object[] = JSON.parse(process.argv[3])
const ACC_A: LocalAccount = ACC[0] as LocalAccount
const ACC_B: LocalAccount = ACC[1] as LocalAccount
const ACC_C: LocalAccount = ACC[2] as LocalAccount
const ACC_D: LocalAccount = ACC[3] as LocalAccount

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const SLEEP_TIME = 1000

async function run_tests() {
  const client_a = await build_client(ACC_A.mnemonic, APIURL)
  const { factory, sienna_token } = await setup(client_a, process.argv[2], undefined, new NullJsonFileWriter)

  const created_pair = await test_create_exchange(factory, sienna_token)
  await sleep(SLEEP_TIME)

  await test_create_existing_pair_error(factory, created_pair)
  
  const pair_address = await test_query_exchanges(factory, created_pair)

  const exchange = new ExchangeContract(pair_address, client_a)
  await test_get_pair_info(exchange, created_pair)
  await test_get_factory_info(exchange, factory.address)
  await test_get_pool(exchange)

  const snip20 = new Snip20Contract(sienna_token.address, client_a)

  await test_liquidity(exchange, snip20, created_pair)
  await sleep(SLEEP_TIME)

  await test_swap(exchange, snip20, created_pair)
}

async function test_create_exchange(factory: FactoryContract, token_info: ContractInfo): Promise<TokenPair> {
  const pair = new TokenPair({
      native_token: {
        denom: 'uscrt'
      }
    },{
      custom_token: {
        contract_addr: token_info.address,
        token_code_hash: token_info.code_hash
      }
    }
  )
  
  await execute_test(
    'test_create_exchange',
    async () => { await factory.create_exchange(pair) }
  )

  return pair
}

async function test_create_existing_pair_error(factory: FactoryContract, pair: TokenPair) {
  await execute_test_expect(
    'test_create_existing_pair_error',
    async () => { await factory.create_exchange(pair) },
    'Pair already exists'
  )

  await sleep(SLEEP_TIME)

  const swapped = new TokenPair(pair.token_1, pair.token_0)

  await execute_test_expect(
    'test_create_existing_pair_error_swapped',
    async () => { await factory.create_exchange(swapped) },
    'Pair already exists'
  )
}

async function test_query_exchanges(factory: FactoryContract, pair: TokenPair): Promise<Address> {
  let address = '';

  await execute_test(
    'test_get_exchange_address',
    async () => { 
      const result = await factory.get_exchange_address(pair)
      address = result
    }
  )

  await execute_test(
    'test_list_exchanges',
    async () => { 
      const result = await factory.list_exchanges(new Pagination(0, 30))
      assert_equal(result.length, 1)
      assert_equal(result[0].address, address)
      assert_objects_equal(result[0].pair, pair)
    }
  )

  return address
}

async function test_get_pair_info(exchange: ExchangeContract, pair: TokenPair) {
  await execute_test(
    'test_get_pair_info',
    async () => {
      const result = await exchange.get_pair_info()
      assert_objects_equal(pair, result)
    }
  )
}

async function test_get_factory_info(exchange: ExchangeContract, address: Address) {
  await execute_test(
    'test_get_factory_info',
    async () => {
      const result = await exchange.get_factory_info()
      assert_equal(address, result.address)
    }
  )
}

async function test_get_pool(exchange: ExchangeContract) {
  await execute_test(
    'test_get_pool',
    async () => {
      const result = await exchange.get_pool()
      assert_equal(result.amount_0, '0')
      assert_equal(result.amount_1, '0')
    }
  )
}

async function test_liquidity(exchange: ExchangeContract, snip20: Snip20Contract, pair: TokenPair) {
  const amount = '5000000'

  // TODO: The current snip20 implementation is garbage and doesn't implement
  // decimal conversion, so providing only a single amount for now
  //const amount1 = '5000000000000000000'

  await snip20_deposit(snip20, amount, exchange.address)

  const token_amount = new TokenPairAmount(pair, amount, amount)

  await execute_test(
    'test_provide_liquidity',
    async () => {
      const result = await exchange.provide_liquidity(token_amount)
      assert_equal(extract_log_value(result, 'share'), amount) //LP tokens
    }
  )

  await execute_test(
    'test_provide_liquidity_pool_not_empty',
    async () => {
      const result = await exchange.get_pool()

      assert_equal(result.amount_0, amount)
      assert_equal(result.amount_1, amount)
    }
  )

  await sleep(SLEEP_TIME)

  await execute_test(
    'test_withdraw_liquidity',
    async () => {
      const result = await exchange.withdraw_liquidity(amount, exchange.signing_client.senderAddress)

      assert_equal(extract_log_value(result, 'withdrawn_share'), amount)
      assert_equal(result.logs[0].events[1].attributes[0].value, exchange.signing_client.senderAddress)
    }
  )

  await sleep(SLEEP_TIME)

  await execute_test(
    'test_pool_empty_after_withdraw',
    async () => {
      const result = await exchange.get_pool()
      
      assert_equal(result.amount_0, '0')
      assert_equal(result.amount_1, '0')
    }
  )
}

async function test_swap(exchange: ExchangeContract, snip20: Snip20Contract, pair: TokenPair) {
  const amount = '5000000'

  // Setup liquidity pool
  await snip20_deposit(snip20, amount, exchange.address)

  const pair_amount = new TokenPairAmount(pair, amount, amount)
  await exchange.provide_liquidity(pair_amount)

  await sleep(SLEEP_TIME)

  const client_b = await build_client(ACC_B.mnemonic, APIURL)
  const exchange_b = new ExchangeContract(exchange.address, client_b)
  const snip20_b = new Snip20Contract(snip20.address, client_b)
  
  const offer_token = new TokenTypeAmount(pair.token_0, '6000000') // swap uscrt for sienna

  await execute_test(
    'test_swap_simulation',
    async () => {
      exchange_b.simulate_swap(offer_token)

      const pool = await exchange_b.get_pool()
      
      assert_equal(pool.amount_0, amount)
      assert_equal(pool.amount_1, amount)
    }
  )

  await execute_test(
    'test_swap_from_native',
    async () => {
      const balance_before = parseInt(await get_native_balance(client_b));
      const result = await exchange_b.swap(offer_token)
      const balance_after = parseInt(await get_native_balance(client_b));
      
      assert(balance_before > balance_after) // TODO: calculate exact amount after adding gas parameters

      const pool = await exchange_b.get_pool()

      const amnt = parseInt(amount)
      const amount_0 = parseInt(pool.amount_0)
      const amount_1 = parseInt(pool.amount_1)

      assert(amnt < amount_0)
      assert(amnt > amount_1)

      assert_equal(extract_log_value(result, 'has_sienna'), 'true')

      const sienna_burned = parseInt(extract_log_value(result, 'sienna_burned') as string)
      const return_amount = parseInt(extract_log_value(result, 'return_amount') as string)

      assert(amnt - return_amount - sienna_burned === amount_1)
    }
  )

  await snip20_deposit(snip20_b, amount, exchange.address)
  
  let key = create_viewing_key()
  await snip20_b.set_viewing_key(key)

  await execute_test(
    'test_get_allowance',
    async () => {
      const result = await snip20_b.get_allowance(client_b.senderAddress, exchange.address, key)
      assert_equal(result.allowance, amount)
    }
  )

  await execute_test_expect(
    'test_swap_from_snip20_insufficient_allowance',
    async () => {
      await exchange_b.swap(new TokenTypeAmount(pair.token_1, '99999999999999'))
    },
    'insufficient allowance:'
  )

  await execute_test(
    'test_swap_from_snip20',
    async () => {
      const native_balance_before = parseInt(await get_native_balance(client_b))
      const token_balance_before = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))

      const swap_amount = '3000000'    
      await exchange_b.swap(new TokenTypeAmount(pair.token_1, swap_amount))

      const native_balance_after = parseInt(await get_native_balance(client_b))
      const token_balance_after = parseInt(await snip20_b.get_balance(client_b.senderAddress, key))

      assert(native_balance_after > native_balance_before) // TODO: calculate exact amount after adding gas parameters
      assert(token_balance_before - parseInt(swap_amount) === token_balance_after)
    }
  )
}

async function snip20_deposit(snip20: Snip20Contract, amount: Uint128, exchange_address: Address) {
  await snip20.deposit(amount)
  await sleep(SLEEP_TIME)

  await snip20.increase_allowance(exchange_address, amount)
  await sleep(SLEEP_TIME)
}

async function get_native_balance(client: SigningCosmWasmClient): Promise<string> {
  const account = await client.getAccount() as Account
  return account.balance[0].amount
}

function create_viewing_key(): ViewingKey {
  const rand_bytes = Random.getBytes(32)
  const key = new Sha256(rand_bytes).digest()

  return Buffer.from(key).toString('base64')
}

run_tests().catch(console.log)
