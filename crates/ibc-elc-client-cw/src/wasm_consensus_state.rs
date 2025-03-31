use light_client::types::Any;
use prost::{DecodeError, Message};

pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

#[derive(::prost::Message)]
pub struct WasmConsensusState {
    #[prost(bytes, tag = "1")]
    pub data: Vec<u8>,
}

impl TryFrom<Any> for WasmConsensusState {
    type Error = DecodeError;

    fn try_from(v: Any) -> Result<Self, Self::Error> {
        if WASM_CONSENSUS_STATE_TYPE_URL == v.type_url.as_str() {
            Self::decode(v.value.as_slice())
        } else {
            Err(DecodeError::new("unexpected type url"))
        }
    }
}

impl From<WasmConsensusState> for Any {
    fn from(v: WasmConsensusState) -> Any {
        Any::new(WASM_CONSENSUS_STATE_TYPE_URL.to_owned(), v.encode_to_vec())
    }
}
