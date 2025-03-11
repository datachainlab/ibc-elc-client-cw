use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use light_client::types::Height as LcpHeight;

#[cw_serde]
pub struct Height {
    revision_number: u64,
    revision_height: u64,
}

#[cw_serde]
pub struct GenesisMetadata {
    key: Binary,
    value: Binary,
}

#[cw_serde]
pub struct StatusResponse {
    pub status: String,
}

#[cw_serde]
pub struct ExportMetadataResponse {
    pub genesis_metadata: Vec<GenesisMetadata>,
}

#[cw_serde]
pub struct TimestampAtHeightResponse {
    pub timestamp: u64,
}

#[cw_serde]
pub struct VerifyClientMessageResponse {}

#[cw_serde]
pub struct CheckForMisbehaviourResponse {
    pub found_misbehaviour: bool,
}

#[cw_serde]
pub struct ContractResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heights: Option<Vec<Height>>,
}

impl ContractResult {
    pub fn success() -> Self {
        Self { heights: None }
    }

    pub fn heights(mut self, heights: Vec<LcpHeight>) -> Self {
        let heights = heights
            .into_iter()
            .map(|h| Height {
                revision_number: h.revision_number(),
                revision_height: h.revision_height(),
            })
            .collect();
        self.heights = Some(heights);
        self
    }
}
