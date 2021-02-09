use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total: Uint128,
    pub pools: Vec<Pool>
}
impl Schedule {
    pub fn new () -> Self {
        Self { total: Uint128::zero(), pools: vec![] }
    }

    pub fn from_pools (pools: Vec<Pool>) -> Self {
        Self { total: Uint128::from(pools.iter().map(|x:&Pool|x.total.u128()).sum::<u128>()), pools }
    }

    pub fn validate (&self) -> cosmwasm_std::StdResult<()> { Ok(()) }

    /// Get amount unlocked for address `a` at time `t`
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> Amount {
        for Pool { accounts, .. } in self.pools.iter() {
            for account in accounts.iter() {
                match account.claimable(a, t) {
                    Some(amount) => return amount,
                    None         => continue
                }
            }
        }
        0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    pub total:    Uint128,
    pub accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub amount:     Uint128,
    pub recipients: Vec<Allocation>,
    pub vesting:    Vesting
}
impl Account {
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> Option<Amount> {
        for Allocation { addr, amount } in self.recipients.iter() {
            if addr == a {
                return Some(self.vest((*amount).u128(), t))
            }
        }
        None
    }
    fn vest (&self, amount: Amount, t: Seconds) -> Amount {
        match &self.vesting {
            // Immediate vesting: if the contract has launched,
            // the recipient can claim the entire allocated amount
            Vesting::Immediate {} => amount,

            // Periodic vesting: need to calculate the maximum amount
            // that the user can claim at the given time.
            Vesting::Periodic { interval, start_at, duration, cliff } => {
                let interval = match interval {
                    Interval::Daily   => DAY,
                    Interval::Monthly => MONTH
                };
                // Can't vest before the cliff
                if t < *start_at { return 0 }
                periodic(amount, interval, t - start_at, *duration, *cliff)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    addr:   HumanAddr,
    amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Vesting {
    Immediate {},
    Periodic {
        interval: Interval,
        start_at: Seconds,
        duration: Seconds,
        cliff:    Percentage
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Interval {
    Daily,
    Monthly
}

fn periodic (
    amount: Amount, interval: Seconds, elapsed: Seconds,
    duration: Seconds, cliff: Percentage,
) -> Amount {

    // mutable for clarity:
    let mut vest = 0;

    // start with the cliff amount
    let cliff = cliff as u128;
    if cliff * amount % 100 > 0 { warn_cliff_remainder() }
    let cliff_amount = (cliff * amount / 100) as u128;
    vest += cliff_amount;

    // then for every `interval` since `t_start`
    // add an equal portion of the remaining amount

    // then, from the remaining amount and the number of vestings
    // determine the size of the portion
    let post_cliff_amount = amount - cliff_amount;
    let n_total: u128 = (duration / interval).into();
    if post_cliff_amount % n_total > 0 { warn_vesting_remainder() }
    let portion = post_cliff_amount / n_total;

    // then determine how many vesting periods have elapsed,
    // up to the maximum; `duration - interval` and `1 + n_elapsed`
    // are used to ensure vesting happens at the begginning of an interval
    let t_elapsed = Seconds::min(elapsed, duration - interval);
    let n_elapsed = t_elapsed / interval;
    let n_elapsed: u128 = (1 + n_elapsed).into();
    //if t_elapsed % interval > interval / 2 { n_elapsed += 1; }

    // then add that amount to the cliff amount
    vest += portion * n_elapsed;

    //println!("periodic {}/{}={} -> {}", n_elapsed, n_total, n_elapsed/n_total, vest);
    vest
}

fn warn_cliff_remainder () {
    //println!("WARNING: division with remainder for cliff amount")
}

fn warn_vesting_remainder () {
    //println!("WARNING: division with remainder for vesting amount")
}
