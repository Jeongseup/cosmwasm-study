use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use error::ContractError;
use msg::InstantiateMsg;
mod contract;
mod error;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

// DepsMut : want to change blockchain state
// Deps : don't want to change blockchain state, just query the state
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        Value {} => to_binary(&contract::query::value(deps)?),
    }
}
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use msg::ExecMsg::*;

    match msg {
        Donate {} => contract::exec::donate(deps, info).map_err(ContractError::from),
        Withdraw {} => contract::exec::withdraw(deps, env, info),
    }
}

#[cfg(test)]
mod test {
    use std::{mem, vec};

    use crate::msg::{ExecMsg, QueryMsg, ValueResp};

    use super::*;
    use cosmwasm_std::{coins, Addr, Coin, Empty};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const ATOM: &str = "atom";

    #[test]
    fn query_value() {
        let mut app = App::default(); // blockchain app simulator
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        // making a object to query
        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        // dbg!(&resp);
        assert_eq!(resp, ValueResp { value: 0 });
    }

    #[test]
    fn donate() {
        let mut app = App::default();
        let sender = Addr::unchecked("sender");
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let _resp = app
            .execute_contract(sender, contract_addr.clone(), &ExecMsg::Donate {}, &[])
            .unwrap();

        // for item in resp.events.iter() {
        //     dbg!(item);
        // }

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 0 });
    }

    #[test]
    fn donate_with_funds() {
        let sender = Addr::unchecked("sender");
        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let _ = app
            .execute_contract(
                sender.clone(),
                contract_addr.clone(),
                &ExecMsg::Donate {},
                &coins(10, ATOM),
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(10, ATOM)
        );
    }

    #[test]
    fn withdraw() {
        let owner = Addr::unchecked("owner");
        let sender1 = Addr::unchecked("sender1");
        let sender2 = Addr::unchecked("sender2");

        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender1, coins(10, ATOM))
                .unwrap();

            router
                .bank
                .init_balance(storage, &sender2, coins(5, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let _ = app
            .execute_contract(
                sender1.clone(),
                contract_addr.clone(),
                &ExecMsg::Donate {},
                &coins(10, ATOM),
            )
            .unwrap();

        let _ = app
            .execute_contract(
                sender2.clone(),
                contract_addr.clone(),
                &ExecMsg::Donate {},
                &coins(5, ATOM),
            )
            .unwrap();

        let _ = app
            .execute_contract(
                owner.clone(),
                contract_addr.clone(),
                &ExecMsg::Withdraw {},
                &[],
            )
            .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(owner).unwrap(),
            coins(15, ATOM)
        );
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            vec![]
        );
        assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![]);
    }

    #[test]
    fn unauthorized_withdraw() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(member, contract_addr, &ExecMsg::Withdraw {}, &[])
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: owner.into()
            },
            err.downcast().unwrap()
        );
    }
}
