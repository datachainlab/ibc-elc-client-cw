use light_client::types::proto::ibc::core::client::v1::Height;
use light_client::types::Any;
use prost::{DecodeError, Message};

pub const WASM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientState";

#[derive(::prost::Message)]
pub struct WasmClientState {
    #[prost(bytes, tag = "1")]
    pub data: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub checksum: Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub latest_height: Option<Height>,
}

impl TryFrom<Any> for WasmClientState {
    type Error = DecodeError;

    fn try_from(v: Any) -> Result<Self, Self::Error> {
        if WASM_CLIENT_STATE_TYPE_URL == v.type_url.as_str() {
            Self::decode(v.value.as_slice())
        } else {
            Err(DecodeError::new("unexpected type url"))
        }
    }
}

impl From<WasmClientState> for Any {
    fn from(v: WasmClientState) -> Any {
        Any::new(WASM_CLIENT_STATE_TYPE_URL.to_owned(), v.encode_to_vec())
    }
}
