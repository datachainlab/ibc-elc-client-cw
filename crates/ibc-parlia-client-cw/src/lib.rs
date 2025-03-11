use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use ibc_elc_client_cw::entrypoint::Entrypoint;
use ibc_elc_client_cw::error::ContractError;
use ibc_elc_client_cw::msg::{InstantiateMsg, QueryMsg, SudoMsg};
use light_client::types::Any;
use parlia_elc::client::ParliaLightClient;
use parlia_elc::client_state::ClientState;
use parlia_elc::consensus_state::ConsensusState;

const ACTIVE: &str = "Active";
const FROZEN: &str = "Frozen";

pub struct ParliaEntrypoint;

impl Entrypoint for ParliaEntrypoint {
    type LightClient = ParliaLightClient;

    fn get_status_from_client_state(any_client_state: Any) -> Result<String, ContractError> {
        let client_state: ClientState = any_client_state
            .try_into()
            .map_err(ContractError::generic)?;
        let status = if client_state.frozen { FROZEN } else { ACTIVE };
        Ok(status.to_owned())
    }

    fn get_timestamp_from_consensus_state(any_consensus_state: Any) -> Result<u64, ContractError> {
        let consensus_state: ConsensusState = any_consensus_state
            .try_into()
            .map_err(ContractError::generic)?;
        let timestamp: u64 = consensus_state
            .timestamp
            .as_unix_timestamp_nanos()
            .try_into()?;
        Ok(timestamp)
    }
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ParliaEntrypoint::instantiate(&ParliaLightClient, deps, env, info, msg)
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    ParliaEntrypoint::sudo(&ParliaLightClient, deps, env, msg)
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    ParliaEntrypoint::query(&ParliaLightClient, deps, env, msg)
}
