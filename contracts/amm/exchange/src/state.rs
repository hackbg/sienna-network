use amm_shared::{
    TokenPair,
    fadroma::{
        platform::{
            Api, CanonicalAddr, Extern, HumanAddr,
            Querier, StdResult, Storage, StdError,
            Canonize, Humanize,
            ContractLink,
        },
        storage::{load, save},
        ViewingKey
    }
};

use serde::{Serialize,Deserialize};

const CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config<A: Clone> {
    pub factory_info:  ContractLink<A>,
    pub lp_token_info: ContractLink<A>,
    pub pair:          TokenPair<A>,
    /// The address of the current contract.
    pub contract_addr: A,
    /// Viewing key used for custom SNIP20 tokens.
    pub viewing_key:   ViewingKey,
}

impl Canonize for Config<HumanAddr> {
    type Output = Config<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Config {
            factory_info:  self.factory_info.canonize(api)?,
            lp_token_info: self.lp_token_info.canonize(api)?,
            pair:          self.pair.canonize(api)?,
            contract_addr: self.contract_addr.canonize(api)?,
            viewing_key:   self.viewing_key
        })
    }
}

impl Humanize for Config<CanonicalAddr> {
    type Output = Config<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Config {
            factory_info:  self.factory_info.humanize(api)?,
            lp_token_info: self.lp_token_info.humanize(api)?,
            pair:          self.pair.humanize(api)?,
            contract_addr: self.contract_addr.humanize(api)?,
            viewing_key:   self.viewing_key
        })
    }
}

pub(crate) fn store_config <S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    config: Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config<HumanAddr>> {
    let result: Config<CanonicalAddr> = load(&deps.storage, CONFIG_KEY)?.ok_or(
        StdError::generic_err("Config doesn't exist in storage.")
    )?;
    result.humanize(&deps.api)
}

#[cfg(test)]
mod tests {
    use super::*;
    use amm_shared::fadroma::platform::testing::mock_dependencies;
    use amm_shared::TokenType;

    #[test]
    fn properly_stores_config() -> StdResult<()> {
        let mut deps = mock_dependencies(10, &[]);

        let config = Config {
            factory_info: ContractLink {
                code_hash: "factory_hash".into(),
                address: HumanAddr("factory".into())
            },
            lp_token_info: ContractLink {
                code_hash: "token_hash".into(),
                address: HumanAddr("lp_token".into())
            },
            pair: TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                }
            ),
            contract_addr: HumanAddr("this".into()),
            viewing_key: ViewingKey("vk".into())
        };

        store_config(&mut deps, config.clone())?;

        let result = load_config(&deps)?;

        assert_eq!(config, result);

        Ok(())
    }
}
