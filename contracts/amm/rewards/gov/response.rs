use std::cmp::min;

use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    config::{GovernanceConfig, IGovernanceConfig},
    governance::Governance,
    poll::{IPoll, Poll},
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceResponse {
    Polls {
        polls: Vec<Poll>,
        total: usize,
        total_pages: u64,
    },
    Poll(Poll),
    CreateViewingKey {
        key: ViewingKey,
    },
    VoteStatus {
        variant: VoteType,
        power: Uint128,
    },

    Config(GovernanceConfig),
}
pub trait IGovernanceResponse<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C, take: u64, page: u64, asc: bool) -> StdResult<Self>;
    fn poll(core: &C, id: u64) -> StdResult<Self>;
    fn vote_status(core: &C, poll_id: u64, address: HumanAddr) -> StdResult<Self>;
    fn config(core: &C) -> StdResult<Self>;
}
impl<S, A, Q, C> IGovernanceResponse<S, A, Q, C> for GovernanceResponse
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C, take: u64, page: u64, _asc: bool) -> StdResult<Self> {
        let take = min(take, 10);

        let total = Poll::total(core)?;
        let total_pages = (total + take - 1) / take;

        let start = (page - 1) * take;
        let end = min(start + take, total);

        let mut polls = vec![];
        for index in start + 1..=end {
            polls.push(Poll::get(core, index)?);
        }

        Ok(GovernanceResponse::Polls {
            total: polls.len().into(),
            polls,
            total_pages,
        })
    }
    fn poll(core: &C, id: u64) -> StdResult<GovernanceResponse> {
        let meta = Poll::metadata(core, id)?;

        let poll = Poll {
            creator: Poll::creator(core, id)?,
            expiration: Poll::expiration(core, id)?,
            status: Poll::status(core, id)?,
            current_quorum: Poll::current_quorum(core, id)?,
            id,
            metadata: meta,
        };
        print!("{:?}", poll);
        Ok(GovernanceResponse::Poll(poll))
    }
    fn config(core: &C) -> StdResult<Self> {
        let config = GovernanceConfig::get(core)?;
        Ok(GovernanceResponse::Config(config))
    }

    fn vote_status(core: &C, poll_id: u64, address: HumanAddr) -> StdResult<Self> {
        let vote = Vote::get(core, address, poll_id)?;
        Ok(GovernanceResponse::VoteStatus {
            power: vote.vote_power.into(),
            variant: vote.variant,
        })
    }
}
