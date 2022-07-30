use crate::error::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Api, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order,
    QueryResponse, Response, StdError, StdResult, WasmMsg, wasm_execute, SubMsg,
};
use cw1_whitelist::{
    state::AdminList,
    contract::execute_execute,
};

use cw_utils::nonpayable;
use simple_ica::PacketMsg;

use cw20::{Cw20Coin, Cw20ReceiveMsg};
// use cw20_base::{
//     contract::{
//     execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
//     },
//     state::{MinterData, TokenInfo, TOKEN_INFO},
// };

use crate::ibc::PACKET_LIFETIME;
use crate::msg::{
    AccountInfo, AccountResponse, AdminResponse, ExecuteMsg, InstantiateMsg, ListAccountsResponse,
    QueryMsg,
};
use crate::state::{ACCOUNTS, ADMIN};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // we store the reflect_id for creating accounts later
    let mut adminaccs = map_validate(deps.api, &&msg.admins)?;
    // let sender = deps.api.addr_validate(&info.sender)?;
    adminaccs.insert(0, info.sender.clone());
    let admin = AdminList {
        admins: adminaccs,
        mutable: msg.mutable,
    };
    // let accs = AdminAccounts { admin:  admin};
    ADMIN.save(deps.storage, &admin)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}
pub fn map_validate(api: &dyn Api, admins: &[String]) -> StdResult<Vec<Addr>> {
    admins.iter().map(|addr| api.addr_validate(addr)).collect()
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddAdmins { new_admins } => execute_add_admins(deps, env, info, new_admins),
        ExecuteMsg::SendMsgs { channel_id, msgs } => {
            execute_send_msgs(deps, env, info, channel_id, msgs)
        }
        ExecuteMsg::CheckRemoteBalance { channel_id } => {
            execute_check_remote_balance(deps, env, info, channel_id)
        }
        ExecuteMsg::SendFunds {
            reflect_channel_id,
            transfer_channel_id,
        } => execute_send_funds(deps, env, info, reflect_channel_id, transfer_channel_id),
        ExecuteMsg::ExecuteCosmosMsg { msgs } => execute_cosmos_msgs(deps, env, info, msgs),
    }
}

pub fn execute_cosmos_msgs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    // auth check
    let admin = ADMIN.load(deps.storage)?;
    if !admin.is_admin(&info.sender) {
        return Err(StdError::generic_err("Only admin may send messages").into());
    }

    let wl_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
    let wasm_msg = wasm_execute(env.contract.address, &wl_msg, vec![])?;

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_attribute("action", "execute_cosmos_msg"))
}

pub fn execute_add_admins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admins: Vec<String>,
) -> Result<Response, ContractError> {
    // auth check
    let mut admin = ADMIN.load(deps.storage)?;
    if !admin.is_admin(&info.sender.to_string()) {
        return Err(ContractError::Std(StdError::generic_err(
            "Only admin may send messages",
        )));
    }

    let new_addrs: Vec<_> = new_admins.iter().map(|a| deps.api.addr_validate(a)).collect::<StdResult<_>>()?;
    admin.admins.extend(new_addrs);

    ADMIN.save(deps.storage, &admin)?;

    Ok(Response::new().add_attribute("action", "handle_update_admin"))
}

pub fn execute_send_msgs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    // auth check
    let admin = ADMIN.load(deps.storage)?;
    if !admin.is_admin(&info.sender) {
        return Err(StdError::generic_err("Only admin may send messages").into());
    }
    // ensure the channel exists (not found if not registered)
    ACCOUNTS.load(deps.storage, &channel_id)?;

    // construct a packet to send
    let packet = PacketMsg::Dispatch { msgs };
    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_send_msgs");
    Ok(res)
}

pub fn execute_check_remote_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
) -> Result<Response, ContractError> {
    // auth check
    let admin = ADMIN.load(deps.storage)?;
    if !admin.is_admin(&info.sender) {
        return Err(StdError::generic_err("Only admin may send messages").into());
    }

    // ensure the channel exists (not found if not registered)
    ACCOUNTS.load(deps.storage, &channel_id)?;

    // construct a packet to send
    let packet = PacketMsg::Balances {};
    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_check_remote_balance");
    Ok(res)
}

pub fn execute_send_funds(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    reflect_channel_id: String,
    transfer_channel_id: String,
) -> Result<Response, ContractError> {
    // intentionally no auth check

    // require some funds
    let amount = match info.funds.pop() {
        Some(coin) => coin,
        None => {
            return Err(ContractError::EmptyFund {});
        }
    };
    // if there are any more coins, reject the message
    if !info.funds.is_empty() {
        return Err(ContractError::TooManyCoins { coins: info.funds });
    }

    // load remote account
    let data = ACCOUNTS.load(deps.storage, &reflect_channel_id)?;
    let remote_addr = match data.remote_addr {
        Some(addr) => addr,
        None => return Err(ContractError::UnregisteredChannel(reflect_channel_id)),
    };

    // construct a packet to send
    let msg = IbcMsg::Transfer {
        channel_id: transfer_channel_id,
        to_address: remote_addr,
        amount,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_send_funds");
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Admins {} => to_binary(&query_admins(deps)?),
        QueryMsg::Account { channel_id } => to_binary(&query_account(deps, channel_id)?),
        QueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
    }
}

fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = ACCOUNTS.load(deps.storage, &channel_id)?;
    Ok(account.into())
}

fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let (channel_id, account) = r?;
            Ok(AccountInfo::convert(channel_id, account))
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}

fn query_admins(deps: Deps) -> StdResult<AdminResponse> {
    let AdminList { admins, mutable } = ADMIN.load(deps.storage)?;
    Ok(AdminResponse {
        admins: admins.into_iter().map(|a| a.into()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info, mock_dependencies_with_balance}, coins, BankMsg, Coin, Uint128, from_slice};

    const CREATOR: &str = "creator";
    const SUB_ADMIN: &str = "sub_admin";

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![SUB_ADMIN.to_string()],
            mutable: true,
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let admin = query_admins(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, admin.admins[0]);
    }

    #[test]
    fn test_query_admins() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![SUB_ADMIN.to_string()],
            mutable: true,
        };
        let info = mock_info(CREATOR, &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let exepected_admins = vec![CREATOR, SUB_ADMIN];
        let admin = query_admins(deps.as_ref()).unwrap();
        assert_eq!(exepected_admins, admin.admins);
    }

    #[test]
    fn test_execute_cosmos_msg() {
        let funds = Coin{
            amount: Uint128::new(123456789),
            denom: "uatom".into(),
        };
        let mut deps = mock_dependencies_with_balance(&[funds.clone()]);

        let msg = InstantiateMsg {
            admins: vec![SUB_ADMIN.to_string()],
            mutable: true,
        };

        let mut info = mock_info(CREATOR, &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let cosmos_msg = vec![BankMsg::Send {
            to_address: SUB_ADMIN.into(),
            amount: vec![funds.clone()],
        }
        .into()];
        info = mock_info(CREATOR, &[funds.clone()]);

        let res = execute_cosmos_msgs(deps.as_mut(), mock_env(), info, cosmos_msg.clone());
        let wl_msg =  cw1_whitelist::msg::ExecuteMsg::Execute { msgs: cosmos_msg};
        let expt_msg: Vec<WasmMsg> = vec![WasmMsg::Execute {
            contract_addr: "cosmos2contract".into(),
            msg: to_binary(&wl_msg).unwrap(),
            funds: vec![],
        }]
        .into();
        assert_eq!(
            res.unwrap().messages,
            expt_msg.into_iter().map(SubMsg::new).collect::<Vec<_>>(),
        );
    }

}
