use cosmwasm_std::{Coin, Addr, QuerierWrapper, QueryRequest, BankMsg, Deps, Api, WasmQuery, Binary, CosmosMsg, StdResult, Uint128, WasmMsg, to_binary};
use cw20_base::msg::{ExecuteMsg as CW20ExecuteMsg, QueryMsg as Cw20QueryMsg};
use cosmwasm_bignumber::{Decimal256, Uint256};
use crate::tax::{deduct_tax};
use cw20::BalanceResponse;


/// @dev Helper function which returns a cosmos wasm msg to transfer cw20 tokens to a recepient address 
/// @param recipient : Address to be transferred cw20 tokens to
/// @param token_contract_address : Contract address of the cw20 token to transfer
/// @param amount : Number of tokens to transfer
pub fn build_transfer_cw20_token_msg(recipient: Addr, token_contract_address: String, amount: Uint128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address,
        msg: to_binary(&CW20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount: amount.into(),
        })?,
        funds: vec![],
    }))
}


/// @dev Helper function which returns a cosmos wasm msg to send cw20 tokens to another contract which implements the ReceiveCW20 Hook 
/// @param recipient_contract_addr : Contract Address to be transferred cw20 tokens to
/// @param token_contract_address : Contract address of the cw20 token to transfer
/// @param amount : Number of tokens to transfer
/// @param msg_ : ExecuteMsg coded into binary which needs to be handled by the recepient contract 
pub fn build_send_cw20_token_msg(recipient_contract_addr: String, token_contract_address: String, amount: Uint128, msg_:Binary) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address,
        msg: to_binary(&CW20ExecuteMsg::Send {
            contract: recipient_contract_addr,
            amount: amount.into(),
            msg: msg_ 
        })?,
        funds: vec![],
    }))
}



/// @dev Helper function which returns a cosmos wasm msg to send native tokens to recepient
/// @param recipient : Contract Address to be transferred native tokens to
/// @param denom : Native token to transfer
/// @param amount : Number of tokens to transfer
pub fn build_send_native_asset_msg( deps: Deps, recipient: Addr, denom: &str, amount: Uint256) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.to_string(),
                amount: amount.into(),
            },
        )?],
    }))
}




/// Used when unwrapping an optional address sent in a contract call by a user.
/// Validates addreess if present, otherwise uses a given default value.
pub fn option_string_to_addr( api: &dyn Api, option_string: Option<String>, default: Addr) -> StdResult<Addr> {
    match option_string {
        Some(input_addr) => api.addr_validate(&input_addr),
        None => Ok(default),
    }
}


// native coins
pub fn get_denom_amount_from_coins(coins: &[Coin], denom: &str) -> Uint256 {
    coins
        .iter()
        .find(|c| c.denom == denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero)
}

// CW20
pub fn cw20_get_balance(querier: &QuerierWrapper, token_address: Addr, balance_address: Addr) -> StdResult<Uint128> {
    let query: BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_address.into(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: balance_address.into(),
        })?,
    }))?;

    Ok(query.balance)
}



pub fn zero_address() -> Addr {
    Addr::unchecked("")
}
