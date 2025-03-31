use cosmwasm_std::StdError;
use ibc::core::ics02_client::error::ClientError;
use light_client::types::proto::protobuf::Error as ProtoError;
use light_client::types::{ClientId, Height};
use light_client::{Error as LightError, LightClientSpecificError};
use prost::{DecodeError, EncodeError};
use std::fmt::{Debug, Display, Formatter, Result};
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ContractError {
    Std(StdError),
    Proto(ProtoError),
    Light(LightError),
    Decode(DecodeError),
    Encode(EncodeError),
    FromUtf8(FromUtf8Error),
    Client(ClientError),
    TryFromInt(TryFromIntError),
    Unsupported(String),
    Generic(String),
}

impl ContractError {
    pub fn generic(msg: impl ToString) -> Self {
        Self::Generic(msg.to_string())
    }

    pub fn unsupported(msg: impl ToString) -> Self {
        Self::Unsupported(msg.to_string())
    }
}

impl Display for ContractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Std(e) => write!(f, "ContractError::Std({})", e),
            Self::Proto(e) => write!(f, "ContractError::Proto({})", e),
            Self::Light(e) => write!(f, "ContractError::Light({})", e),
            Self::Decode(e) => write!(f, "ContractError::Decode({})", e),
            Self::Encode(e) => write!(f, "ContractError::Encode({})", e),
            Self::FromUtf8(e) => write!(f, "ContractError::FromUtf8({})", e),
            Self::Client(e) => write!(f, "ContractError::Client({})", e),
            Self::TryFromInt(e) => write!(f, "ContractError::TryFromInt({})", e),
            Self::Unsupported(e) => write!(f, "ContractError::Unsupported({})", e),
            Self::Generic(e) => write!(f, "ContractError::Generic({})", e),
        }
    }
}

impl From<StdError> for ContractError {
    fn from(v: StdError) -> Self {
        Self::Std(v)
    }
}

impl From<ProtoError> for ContractError {
    fn from(v: ProtoError) -> Self {
        Self::Proto(v)
    }
}

impl From<LightError> for ContractError {
    fn from(v: LightError) -> Self {
        Self::Light(v)
    }
}

impl From<DecodeError> for ContractError {
    fn from(v: DecodeError) -> Self {
        Self::Decode(v)
    }
}

impl From<EncodeError> for ContractError {
    fn from(v: EncodeError) -> Self {
        Self::Encode(v)
    }
}

impl From<FromUtf8Error> for ContractError {
    fn from(v: FromUtf8Error) -> Self {
        Self::FromUtf8(v)
    }
}

impl From<ClientError> for ContractError {
    fn from(v: ClientError) -> Self {
        Self::Client(v)
    }
}

impl From<TryFromIntError> for ContractError {
    fn from(v: TryFromIntError) -> Self {
        Self::TryFromInt(v)
    }
}

#[derive(Debug)]
pub enum WasmLightClientSpecificError {
    NotAnyWasmClientState(DecodeError, ClientId),
    NotWasmClientState(DecodeError, ClientId),
    NotAnyClientState(DecodeError, ClientId),

    NotAnyWasmConsensusState(DecodeError, ClientId, Height),
    NotWasmConsensusState(DecodeError, ClientId, Height),
    NotAnyConsensusState(DecodeError, ClientId, Height),
}

impl Display for WasmLightClientSpecificError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(self, f)
    }
}

impl LightClientSpecificError for WasmLightClientSpecificError {}
