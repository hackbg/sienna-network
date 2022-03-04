use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::poll::Poll;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GovernanceConfig {
    pub threshold: Option<Uint128>,
    pub quorum: Option<Decimal>,
    pub deadline: Option<u64>,
}
impl GovernanceConfig {
    //metadata configuration
    pub const MIN_TITLE_LENGTH: usize = 8;
    pub const MAX_TITLE_LENGTH: usize = 64;

    pub const MIN_DESC_LENGTH: usize = 8;
    pub const MAX_DESC_LENGTH: usize = 1024;

    pub const MIN_POLL_TYPE_LENGTH: usize = 8;
    pub const MAX_POLL_TYPE_LENGTH: usize = 24;

    pub const DEFAULT_QUORUM_PERCENT: u64 = 33;
    pub const DEFAULT_TRESHOLD: Uint128 = Uint128(3500);
    pub const DEFAULT_DEADLINE: u64 = 7 * 24 * 60 * 60;

    //storage keys
    pub const THRESHOLD: &'static [u8] = b"/gov/threshold";
    pub const QUORUM: &'static [u8] = b"/gov/quorum";
    pub const DEADLINE: &'static [u8] = b"/gov/deadline";
    pub const VOTES: &'static [u8] = b"/gov/votes";

    pub const USER_POLLS: &'static [u8] = b"/gov/user_polls";
}
pub trait IGovernanceConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn get(core: &C) -> StdResult<Self>;
    fn threshold(core: &C) -> StdResult<Uint128>;
    fn quorum(core: &C) -> StdResult<Decimal>;
    fn deadline(core: &C) -> StdResult<u64>;
}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn initialize(&mut self, core: &mut C, _env: &Env) -> StdResult<Vec<CosmosMsg>> {
        core.set(Poll::TOTAL, 0)?;
        self.store(core)
    }
    fn get(core: &C) -> StdResult<Self> {
        Ok(Self {
            deadline: Some(Self::deadline(core)?),
            quorum: Some(Self::quorum(core)?),
            threshold: Some(Self::threshold(core)?),
        })
    }

    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let GovernanceConfig {
            deadline,
            threshold,
            quorum,
        } = self;
        if let Some(deadline) = deadline {
            core.set(Self::DEADLINE, deadline)?;
        }
        if let Some(threshold) = threshold {
            core.set(Self::THRESHOLD, threshold)?;
        }
        if let Some(quorum) = quorum {
            core.set(Self::QUORUM, quorum)?;
        }
        Ok(vec![])
    }

    fn threshold(core: &C) -> StdResult<Uint128> {
        Ok(core.get::<Uint128>(Self::THRESHOLD)?.unwrap())
    }

    fn quorum(core: &C) -> StdResult<Decimal> {
        Ok(core.get::<Decimal>(Self::QUORUM)?.unwrap())
    }

    fn deadline(core: &C) -> StdResult<u64> {
        Ok(core.get::<u64>(Self::DEADLINE)?.unwrap())
    }
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            threshold: Some(Self::DEFAULT_TRESHOLD),
            quorum: Some(Decimal::percent(Self::DEFAULT_QUORUM_PERCENT)),
            deadline: Some(Self::DEFAULT_DEADLINE),
        }
    }
}
