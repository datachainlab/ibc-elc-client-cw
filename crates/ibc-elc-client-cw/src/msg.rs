use crate::error::ContractError;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use light_client::types::Height as LcpHeight;

// ------------------------------------------------------------
// Implementation of the InstantiateMsg struct
// ------------------------------------------------------------

#[cw_serde]
pub struct InstantiateMsg {
    pub client_state: Binary,
    pub consensus_state: Binary,
    pub checksum: Binary,
}

// ------------------------------------------------------------
// Implementation of the SudoMsg enum and its variants
// ------------------------------------------------------------

#[cw_serde]
pub enum SudoMsg {
    UpdateState(UpdateStateMsg),
    UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsg),
    VerifyUpgradeAndUpdateState(VerifyUpgradeAndUpdateStateMsg),
    VerifyMembership(VerifyMembershipMsg),
    VerifyNonMembership(VerifyNonMembershipMsg),
    MigrateClientStore(MigrateClientStoreMsg),
}

#[cw_serde]
pub struct UpdateStateMsg {
    pub client_message: Binary,
}

#[cw_serde]
pub struct UpdateStateOnMisbehaviourMsg {
    pub client_message: Binary,
}

#[cw_serde]
pub struct VerifyUpgradeAndUpdateStateMsg {
    pub upgrade_client_state: Binary,
    pub upgrade_consensus_state: Binary,
    pub proof_upgrade_client: Binary,
    pub proof_upgrade_consensus_state: Binary,
}

#[cw_serde]
pub struct MerklePath {
    pub key_path: Vec<Binary>,
}

impl MerklePath {
    pub fn prefix_and_path(&self) -> Result<(Vec<u8>, String), ContractError> {
        let (prefix, path) = self.key_path.split_first().ok_or(ContractError::generic(
            "prefix not found in the merkle path",
        ))?;
        let prefix = prefix.to_vec();
        let path = path.iter().map(|b| b.to_vec()).collect::<Vec<_>>();
        let path = path.concat();
        let path = String::from_utf8(path)?;
        Ok((prefix, path))
    }
}

#[cw_serde]
pub struct Height {
    #[serde(default)]
    pub revision_number: u64,
    #[serde(default)]
    pub revision_height: u64,
}

impl From<Height> for LcpHeight {
    fn from(v: Height) -> Self {
        Self::new(v.revision_number, v.revision_height)
    }
}

#[cw_serde]
pub struct VerifyMembershipMsg {
    pub height: Height,
    pub delay_time_period: u64,
    pub delay_block_period: u64,
    pub proof: Binary,
    pub merkle_path: MerklePath,
    pub value: Binary,
}

#[cw_serde]
pub struct VerifyNonMembershipMsg {
    pub height: Height,
    pub delay_time_period: u64,
    pub delay_block_period: u64,
    pub proof: Binary,
    pub merkle_path: MerklePath,
}

#[cw_serde]
pub struct MigrateClientStoreMsg {}

// ------------------------------------------------------------
// Implementation of the QueryMsg enum and its variants
// ------------------------------------------------------------

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(crate::response::StatusResponse)]
    Status(StatusMsg),
    #[returns(crate::response::ExportMetadataResponse)]
    ExportMetadata(ExportMetadataMsg),
    #[returns(crate::response::TimestampAtHeightResponse)]
    TimestampAtHeight(TimestampAtHeightMsg),
    #[returns(crate::response::VerifyClientMessageResponse)]
    VerifyClientMessage(VerifyClientMessageMsg),
    #[returns(crate::response::CheckForMisbehaviourResponse)]
    CheckForMisbehaviour(CheckForMisbehaviourMsg),
}

#[cw_serde]
pub struct StatusMsg {}

#[cw_serde]
pub struct ExportMetadataMsg {}

#[cw_serde]
pub struct TimestampAtHeightMsg {
    pub height: Height,
}

#[cw_serde]
pub struct VerifyClientMessageMsg {
    pub client_message: Binary,
}

#[cw_serde]
pub struct CheckForMisbehaviourMsg {
    pub client_message: Binary,
}
