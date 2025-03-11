use cosmwasm_std::StdError;
use ibc::core::ics02_client::error::ClientError;
use light_client::types::proto::protobuf::Error as ProtoError;
use light_client::Error as LightError;
use prost::{DecodeError, EncodeError};
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

impl ToString for ContractError {
    fn to_string(&self) -> String {
        match self {
            Self::Std(e) => e.to_string(),
            Self::Proto(e) => e.to_string(),
            Self::Light(e) => e.to_string(),
            Self::Decode(e) => e.to_string(),
            Self::Encode(e) => e.to_string(),
            Self::FromUtf8(e) => e.to_string(),
            Self::Client(e) => e.to_string(),
            Self::Unsupported(e) => format!("ContractError::Unsupported error {}", e),
            Self::Generic(e) => format!("ContractError::Generic error {}", e),
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
