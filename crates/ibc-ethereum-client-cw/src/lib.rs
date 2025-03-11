use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use ethereum_elc::client::EthereumLightClient;
use ethereum_elc::ibc::{client_state::ClientState, consensus_state::ConsensusState};
use ibc::core::ics02_client::client_state::ClientState as _;
use ibc_elc_client_cw::entrypoint::Entrypoint;
use ibc_elc_client_cw::error::ContractError;
use ibc_elc_client_cw::msg::{InstantiateMsg, QueryMsg, SudoMsg};
use light_client::types::Any;

const SYNC_COMMITTEE_SIZE: usize = if cfg!(feature = "minimal") {
    ethereum_elc::ibc::consensus::preset::minimal::PRESET.SYNC_COMMITTEE_SIZE
} else {
    ethereum_elc::ibc::consensus::preset::mainnet::PRESET.SYNC_COMMITTEE_SIZE
};

const ACTIVE: &str = "Active";
const FROZEN: &str = "Frozen";

struct EthereumEntrypoint;

impl Entrypoint for EthereumEntrypoint {
    type LightClient = EthereumLightClient<SYNC_COMMITTEE_SIZE>;

    fn get_status_from_client_state(any_client_state: Any) -> Result<String, ContractError> {
        let client_state: ClientState<SYNC_COMMITTEE_SIZE> =
            any_client_state.to_proto().try_into()?;
        let status = if client_state.is_frozen() {
            FROZEN
        } else {
            ACTIVE
        };
        Ok(status.to_owned())
    }

    fn get_timestamp_from_consensus_state(any_consensus_state: Any) -> Result<u64, ContractError> {
        let consensus_state: ConsensusState = any_consensus_state.to_proto().try_into()?;
        Ok(consensus_state.timestamp.nanoseconds())
    }
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    EthereumEntrypoint::instantiate(
        &EthereumLightClient::<SYNC_COMMITTEE_SIZE>,
        deps,
        env,
        info,
        msg,
    )
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    EthereumEntrypoint::sudo(&EthereumLightClient::<SYNC_COMMITTEE_SIZE>, deps, env, msg)
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    EthereumEntrypoint::query(&EthereumLightClient::<SYNC_COMMITTEE_SIZE>, deps, env, msg)
}
