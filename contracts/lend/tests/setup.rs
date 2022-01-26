use lend_shared::fadroma::{
    decimal::one_token,
    ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv},
    from_binary,
    snip20_impl,
    snip20_impl::msg::{
        InitMsg as Snip20InitMsg,
        HandleMsg as Snip20HandleMsg,
        InitConfig as Snip20InitConfig,
        InitialBalance
    },
    to_binary, Binary, Composable, ContractLink, Decimal256, Env, HandleResponse, HumanAddr,
    InitResponse, Permit, StdError, StdResult, Uint128, Uint256,
};

use lend_shared::interfaces::{interest_model, market, overseer};
use overseer::MarketInitConfig;

use crate::{impl_contract_harness_default, ADMIN};
use lend_interest_model;
use lend_market;
use lend_oracle;
use lend_overseer;

use lend_oracle::SourceQuery;

pub struct Token;
impl ContractHarness for Token {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        snip20_impl::snip20_init(deps, env, from_binary(&msg)?, snip20_impl::DefaultSnip20Impl)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        snip20_impl::snip20_handle(deps, env, from_binary(&msg)?, snip20_impl::DefaultSnip20Impl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        snip20_impl::snip20_query(deps, from_binary(&msg)?, snip20_impl::DefaultSnip20Impl)
    }
}

pub struct Overseer;
impl_contract_harness_default!(Overseer, lend_overseer);

pub struct Oracle;
impl_contract_harness_default!(Oracle, lend_oracle);

pub struct Market;
impl_contract_harness_default!(Market, lend_market);

pub struct InterestModel;
impl_contract_harness_default!(InterestModel, lend_interest_model);

pub struct MockBand;

impl ContractHarness for MockBand {
    fn init(&self, _deps: &mut MockDeps, _env: Env, _msg: Binary) -> StdResult<InitResponse> {
        Ok(InitResponse::default())
    }

    fn handle(&self, _deps: &mut MockDeps, _env: Env, _msg: Binary) -> StdResult<HandleResponse> {
        Err(StdError::GenericErr {
            msg: "Not Implemented".to_string(),
            backtrace: None,
        })
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let msg = from_binary(&msg).unwrap();
        match msg {
            SourceQuery::GetReferenceData {
                base_symbol, ..
            } => {
                let key: &[u8] = base_symbol.as_bytes();
                match deps.get(key).unwrap() {
                    Some(value) => to_binary(&lend_oracle::BandResponse {
                        rate: value,
                        last_updated_base: 1628544285u64,
                        last_updated_quote: 3377610u64,
                    }),
                    None => to_binary(&lend_oracle::BandResponse {
                        rate: Uint128(1_000_000_000_000_000_000),
                        last_updated_base: 1628544285u64,
                        last_updated_quote: 3377610u64,
                    }),
                }
            }
            SourceQuery::GetReferenceDataBulk {
                base_symbols, ..
            } => {
                let mut results = Vec::new();
                let data = lend_oracle::BandResponse {
                    rate: Uint128(1_000_000),
                    last_updated_base: 1628544285u64,
                    last_updated_quote: 3377610u64,
                };

                for _ in base_symbols {
                    results.push(data.clone());
                }
                to_binary(&results)
            }
        }
    }
}

pub struct Lend {
    pub ensemble: ContractEnsemble,
    pub overseer: ContractLink<HumanAddr>,
    pub underlying_token_one: ContractLink<HumanAddr>,
    pub underlying_token_two: ContractLink<HumanAddr>,
    pub underlying_token_three: ContractLink<HumanAddr>,
    pub interest_model: ContractLink<HumanAddr>,
    pub mock_band: ContractLink<HumanAddr>,
}

impl Lend {
    pub fn new(
        market: Option<Box<dyn ContractHarness>>,
        overseer: Option<Box<dyn ContractHarness>>,
    ) -> Self {
        let mut ensemble = ContractEnsemble::new(50);

        let overseer = ensemble.register(overseer.unwrap_or(Box::new(Overseer)));
        let market = ensemble.register(market.unwrap_or(Box::new(Market)));
        let oracle = ensemble.register(Box::new(Oracle));
        let mock_band = ensemble.register(Box::new(MockBand));
        let token = ensemble.register(Box::new(Token));
        let interest = ensemble.register(Box::new(InterestModel));

        let decimals = 6;
        let token_config = Snip20InitConfig::builder().enable_mint().build();

        let underlying_token_one = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: "Underlying Token".into(),
                    admin: None,
                    symbol: "EDNO".into(),
                    decimals,
                    initial_allowances: None,
                    initial_balances: Some(vec![
                        InitialBalance {
                            address: ADMIN.into(),
                            amount: Uint128(1000 * one_token(decimals)),
                        },
                        InitialBalance {
                            address: "borrower".into(),
                            amount: Uint128(5 * one_token(decimals)),
                        },
                    ]),
                    prng_seed: Binary::from(b"whatever"),
                    config: Some(token_config.clone()),
                    callback: None,
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "underlying_one".into(),
                        code_hash: token.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let underlying_token_two = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: "Underlying Token".into(),
                    admin: None,
                    symbol: "DVE".into(),
                    decimals: 3,
                    initial_allowances: None,
                    initial_balances: Some(vec![
                        InitialBalance {
                            address: ADMIN.into(),
                            amount: Uint128(one_token(3)),
                        },
                        InitialBalance {
                            address: "borrower".into(),
                            amount: Uint128(5 * one_token(3)),
                        },
                    ]),
                    prng_seed: Binary::from(b"whatever"),
                    config: Some(token_config.clone()),
                    callback: None,
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "underlying_two".into(),
                        code_hash: token.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let underlying_token_three = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: "Underlying Token".into(),
                    admin: None,
                    symbol: "TRI".into(),
                    decimals,
                    initial_allowances: None,
                    initial_balances: Some(vec![
                        InitialBalance {
                            address: ADMIN.into(),
                            amount: Uint128(one_token(decimals)),
                        },
                        InitialBalance {
                            address: "borrower".into(),
                            amount: Uint128(5 * one_token(decimals)),
                        },
                    ]),
                    prng_seed: Binary::from(b"whatever"),
                    config: Some(token_config),
                    callback: None,
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "underlying_three".into(),
                        code_hash: token.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let mock_band = ensemble
            .instantiate(
                mock_band.id,
                &{},
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "mock_band".into(),
                        code_hash: mock_band.code_hash,
                    },
                ),
            )
            .unwrap();

        let interest_model = ensemble
            .instantiate(
                interest.id,
                &interest_model::InitMsg {
                    admin: None,
                    base_rate_year: Decimal256::zero(),
                    multiplier_year: Decimal256::one(),
                    jump_multiplier_year: Decimal256::zero(),
                    jump_threshold: Decimal256::zero(),
                    blocks_year: Some(6311520),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "interest_model".into(),
                        code_hash: interest.code_hash,
                    },
                ),
            )
            .unwrap();

        let overseer = ensemble
            .instantiate(
                overseer.id,
                &overseer::InitMsg {
                    admin: None,
                    prng_seed: Binary::from(b"whatever"),
                    close_factor: Decimal256::from_uint256(51000000000000000u128).unwrap(),
                    premium: Decimal256::one(),
                    market_contract: market,
                    oracle_contract: oracle,
                    oracle_source: mock_band.clone(),
                    entropy: Binary::from(b"whatever"),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "overseer".into(),
                        code_hash: overseer.code_hash,
                    },
                ),
            )
            .unwrap();

        Self {
            ensemble,
            overseer,
            underlying_token_one,
            underlying_token_two,
            underlying_token_three,
            interest_model,
            mock_band,
        }
    }

    pub fn get_liquidity(
        &self,
        market: Option<HumanAddr>,
        redeem_amount: Uint256,
        borrow_amount: Uint256,
        block: Option<u64>,
    ) -> StdResult<overseer::AccountLiquidity> {
        self.ensemble.query(
            self.overseer.address.clone(),
            overseer::QueryMsg::AccountLiquidity {
                method: Permit::<overseer::OverseerPermissions>::new(
                    "borrower",
                    vec![overseer::OverseerPermissions::AccountInfo],
                    vec![self.overseer.address.clone()],
                    "balance",
                )
                .into(),
                market,
                redeem_amount,
                borrow_amount,
                block,
            },
        )
    }

    pub fn get_markets(&self) -> StdResult<Vec<overseer::Market<HumanAddr>>> {
        self.ensemble.query(
            self.overseer.address.clone(),
            overseer::QueryMsg::Markets {
                pagination: overseer::Pagination {
                    start: 0,
                    limit: 30,
                },
            },
        )
    }

    pub fn whitelist_market(
        &mut self,
        underlying_asset: ContractLink<HumanAddr>,
        ltv_ratio: Decimal256,
        exchange_rate: Option<Decimal256>
    ) -> StdResult<overseer::Market<HumanAddr>> {
        self.ensemble.execute(
            &overseer::HandleMsg::Whitelist {
                config: MarketInitConfig {
                    prng_seed: Binary::from(b"seed_for_sienna_market"),
                    underlying_asset,
                    ltv_ratio,
                    config: market::Config {
                        initial_exchange_rate: exchange_rate.unwrap_or(Decimal256::one()),
                        reserve_factor: Decimal256::one(),
                        seize_factor: Decimal256::one(),
                    },
                    interest_model_contract: self.interest_model.clone(),
                },
            },
            MockEnv::new(ADMIN, self.overseer.clone()),
        )?;

        Ok(self.get_markets().unwrap().pop().unwrap())
    }

    pub fn set_oracle_price(&mut self, key: &[u8], value: Uint128) -> StdResult<()> {
        self.ensemble
            .deps_mut(self.mock_band.address.clone(), |s| {
                s.set(key, value).unwrap();
            })
            .unwrap();

        Ok(())
    }

    pub fn prefund_user(
        &mut self,
        address: impl Into<HumanAddr>,
        amount: Uint128,
        token: ContractLink<HumanAddr>
    ) {
        self.ensemble.execute(
            &Snip20HandleMsg::Mint {
                recipient: address.into(),
                amount,
                memo: None,
                padding: None
            },
            MockEnv::new(ADMIN, token)
        ).unwrap()
    }

    pub fn prefund_and_deposit(
        &mut self,
        address: impl Into<HumanAddr>,
        amount: Uint128,
        market: HumanAddr
    ) {
        let token: ContractLink<HumanAddr> = self.ensemble.query(
            market.clone(),
            market::QueryMsg::UnderlyingAsset {}
        ).unwrap();

        self.prefund_user(address, amount, token.clone());

        self.ensemble.execute(
            &Snip20HandleMsg::Send {
                recipient: market,
                recipient_code_hash: None,
                amount,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
                memo: None,
                padding: None
            },
            MockEnv::new(ADMIN, token)
        ).unwrap()
    }
}

impl Default for Lend {
    fn default() -> Self {
        Lend::new(None, None)
    }
}
