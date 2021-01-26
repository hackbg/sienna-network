use cosmwasm_std::{
    coins, from_binary,
    StdResult, StdError,
    HumanAddr, Coin,
    Extern, MemoryStorage
};

use cosmwasm_std::testing::{
    mock_dependencies_with_balances, /*mock_env,*/
    MockApi, MockQuerier
};

type ExternMock = Extern<MemoryStorage, MockApi, MockQuerier>;
type HandleResult = cosmwasm_std::StdResult<cosmwasm_std::HandleResponse>;

pub fn harness (balances: &[(&HumanAddr, &[Coin])])-> ExternMock {
    let mut deps = mock_dependencies_with_balances(20, &balances);

    // As the admin
    // When I init the contract
    // Then I want to be able to query its state
    let res = mgmt::init(
        &mut deps,
        mock_env(0, 0, balances[0].0, balances[0].1.into()),
        mgmt::msg::Init { token: None }
    ).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

pub fn mock_env (
    height: u64, time: u64, sender: &HumanAddr, sent_funds: Vec<Coin>
) -> Env {
    block: BlockInfo { height, time, chain_id: "secret".into() },
    message: MessageInfo { sender: sender.into(), sent_funds },
    contract: ContractInfo { address: "contract".into() },
    contract_key: Some("".into()),
    contract_code_hash: "0".into()
}

pub fn tx (
    mut deps: ExternMock,
    env:      Env,
    tx:       mgmt::contract::Handle
) -> HandleResult {
    mgmt::handle(&mut deps, env, tx)
}

macro_rules! query {
    (
        $Query:ident ( $deps:ident ) -> $Response:ident ( $($arg:ident),* )
        $Assertions:block
    ) => {
        let response = from_binary(
            &mgmt::query(&$deps, mgmt::msg::Query::$Query {}).unwrap()
        ).unwrap();
        match response {
            mgmt::msg::Response::$Response {$($arg),*} => {
                $Assertions
            },
            _ => panic!("{} returned something other than {}",
                stringify!($Query), stringify!($Response))
        }
    }
}

macro_rules! tx {
    (
        $deps:expr, $env:expr,
        $Msg:ident $({ $($arg:ident : $val:expr),* })?
    ) => { {
        mgmt::handle(
            &mut $deps,
            $env,
            mgmt::msg::Handle::$Msg { $($($arg:$val)*)? }
        )
    } }
}

macro_rules! canon {
    ($deps:ident, $($x:tt)*) => {
        $deps.api.canonical_address($($x)*).unwrap();
    }
}
