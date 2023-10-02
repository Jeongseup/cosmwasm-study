use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

mod contract;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    contract::instantiate(deps)
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
    _env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> StdResult<Response> {
    use msg::ExecMsg::*;

    match msg {
        Poke {} => contract::exec::poke(deps, info),
    }
}

#[cfg(test)]
mod test {
    use crate::msg::{ExecMsg, QueryMsg, ValueResp};

    use super::*;
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_value() {
        let mut app = App::default(); // blockchain app simulator
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &QueryMsg::Value {},
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
    fn poke() {
        let mut app = App::default();
        let sender = Addr::unchecked("sender");
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &QueryMsg::Value {},
                &[],
                "Counting Contract",
                None,
            )
            .unwrap();

        let resp = app
            .execute_contract(sender, contract_addr.clone(), &ExecMsg::Poke {}, &[])
            .unwrap();

        // for item in resp.events.iter() {
        //     dbg!(item);
        // }
        dbg!(resp);

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }
}
