mod state;

use serde::{Serialize, Deserialize};
use lend_shared::fadroma::{
    schemars, schemars::JsonSchema,
    admin,
    admin::{Admin, assert_admin},
    Callback,
    cosmwasm_std,
    derive_contract::*,
    HumanAddr, InitResponse, HandleResponse,
    QueryRequest, StdResult, WasmQuery, CosmosMsg,
    WasmMsg, log, to_binary,
    ContractLink, Decimal256
};
use lend_shared::interfaces::oracle::{
    PriceResponse, PricesResponse,
    SourceQuery, PriceAsset
};

use state::{Contracts, Asset};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BandResponse {
    pub rate: cosmwasm_std::Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::oracle",
    component(path = "admin")
)]
pub trait BandOracleConsumer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        source: ContractLink<HumanAddr>,
        initial_assets: Vec<PriceAsset>,
        callback: Callback<HumanAddr>
    ) -> StdResult<InitResponse> {
        Contracts::save_source(deps, &source)?;
        Contracts::save_overseer(deps, &callback.contract)?;

        for asset in initial_assets {
            Asset::save(deps, &asset)?;
        }

        let mut result = admin::DefaultImpl.new(admin, deps, env)?;
        result.messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: callback.contract.address,
            callback_code_hash: callback.contract.code_hash,
            send: vec![],
            msg: callback.msg
        }));

        Ok(result)
    }

    #[handle]
    fn update_assets(assets: Vec<PriceAsset>) -> StdResult<HandleResponse> {
        if Contracts::load_overseer(deps)?.address != env.message.sender {
            assert_admin(deps, &env)?;
        }

        for asset in assets {
            Asset::save(deps, &asset)?;
        }

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "update_asset")],
            data: None
        })
    }

    #[query("price")]
    fn price(base: HumanAddr, quote: HumanAddr) -> StdResult<PriceResponse> {
        let source = Contracts::load_source(deps)?;

        let res: BandResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: source.address,
            callback_code_hash: source.code_hash,
            msg: to_binary(&SourceQuery::GetReferenceData {
                base_symbol: Asset::load_symbol(deps, &base)?,
                quote_symbol: Asset::load_symbol(deps, &quote)?,
            })?,
        }))?;

        Ok(PriceResponse {
            rate: Decimal256::from_uint256(res.rate)?,
            last_updated_base: res.last_updated_base,
            last_updated_quote: res.last_updated_quote,
        })
    }

    #[query("prices")]
    fn prices(base: Vec<HumanAddr>, quote: Vec<HumanAddr>) -> StdResult<PricesResponse> {
        let source = Contracts::load_source(deps)?;

        let prices: Vec<BandResponse> = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: source.address,
            callback_code_hash: source.code_hash,
            msg: to_binary(&SourceQuery::GetReferenceDataBulk {
                base_symbols: base.iter()
                    .map(|x| Asset::load_symbol(deps, x))
                    .collect::<StdResult<Vec<String>>>()?,
                quote_symbols: quote.iter()
                    .map(|x| Asset::load_symbol(deps, x))
                    .collect::<StdResult<Vec<String>>>()?,
            })?,
        }))?;

        let prices: StdResult<Vec<PriceResponse>> = prices.into_iter().map(|price| 
            Ok(PriceResponse{
                rate: Decimal256::from_uint256(price.rate)?,
                last_updated_base: price.last_updated_base,
                last_updated_quote: price.last_updated_quote,
            })
        ).collect();

        let prices = prices?;

        Ok(PricesResponse { prices })
    }
}
