use crate::error::{ContractError, WasmLightClientSpecificError};
use crate::wasm_client_state::WasmClientState;
use crate::wasm_consensus_state::WasmConsensusState;
use core::str::FromStr;
use cosmwasm_std::{Binary, CustomQuery, Deps, DepsMut, Empty, Env, Storage};
use light_client::types::{Any, ClientId, Height, Time};
use light_client::Error as LightError;
use light_client::{ClientReader, HostClientReader, HostContext};
use prost::Message;
use store::KVStore;

pub const SUBJECT_PREFIX: &[u8] = b"subject/";
pub const SUBSTITUTE_PREFIX: &[u8] = b"substitute/";

pub const CLIENT_PREFIX: &str = "clients";
pub const CLIENT_STATE: &str = "clientState";
pub const CONSENSUS_STATE_PREFIX: &str = "consensusStates";
pub const PROCESSED_TIME: &str = "processedTime";
pub const PROCESSED_HEIGHT: &str = "processedHeight";

pub struct Context<'a, C: CustomQuery = Empty> {
    deps: Option<Deps<'a, C>>,
    deps_mut: Option<DepsMut<'a, C>>,
    env: Env,
    client_id: ClientId,
    checksum: Option<Binary>,
    migration_prefix: Option<&'static [u8]>,
}

impl<'a, C: CustomQuery> Context<'a, C> {
    pub fn new_ref(deps: Deps<'a, C>, env: Env) -> Self {
        let client_id = ClientId::from_str(env.contract.address.as_str()).unwrap();

        Self {
            deps: Some(deps),
            deps_mut: None,
            env,
            client_id,
            checksum: None,
            migration_prefix: None,
        }
    }

    pub fn new_mut(deps_mut: DepsMut<'a, C>, env: Env) -> Self {
        let client_id = ClientId::from_str(env.contract.address.as_str()).unwrap();

        Self {
            deps: None,
            deps_mut: Some(deps_mut),
            env,
            client_id,
            checksum: None,
            migration_prefix: None,
        }
    }

    pub fn storage_ref(&self) -> &dyn Storage {
        match (&self.deps, &self.deps_mut) {
            (Some(ref deps), _) => deps.storage,
            (_, Some(ref deps)) => deps.storage,
            _ => panic!("Either deps or deps_mut should be available"),
        }
    }

    pub fn storage_mut(&mut self) -> &mut dyn Storage {
        match self.deps_mut {
            Some(ref mut deps) => deps.storage,
            None => panic!("deps_mut should be available"),
        }
    }

    pub fn log(&self, msg: &str) -> Option<()> {
        self.deps.map(|deps| deps.api.debug(msg))
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn set_checksum(&mut self, checksum: Binary) {
        self.checksum = Some(checksum);
    }

    pub fn obtain_checksum(&self) -> Result<Binary, ContractError> {
        match &self.checksum {
            Some(checksum) => Ok(checksum.clone()),
            None => {
                let value = self.get_prefixed(CLIENT_STATE.as_bytes())?;
                let any_wasm_client_state = Any::decode(value.as_slice())?;
                let wasm_client_state: WasmClientState = any_wasm_client_state.try_into()?;
                Ok(wasm_client_state.checksum.into())
            }
        }
    }

    pub fn set_subject_prefix(&mut self) {
        self.migration_prefix = Some(SUBJECT_PREFIX);
    }

    pub fn set_substitute_prefix(&mut self) {
        self.migration_prefix = Some(SUBSTITUTE_PREFIX);
    }

    pub fn prefixed_key(&self, key: impl AsRef<[u8]>) -> Vec<u8> {
        let mut prefixed_key = Vec::new();
        prefixed_key.extend_from_slice(self.migration_prefix.unwrap_or(b""));
        prefixed_key.extend_from_slice(key.as_ref());
        prefixed_key
    }

    pub fn get_prefixed(&self, key: impl AsRef<[u8]>) -> Result<Vec<u8>, ContractError> {
        let prefixed_key = self.prefixed_key(key);
        self.storage_ref()
            .get(&prefixed_key)
            .ok_or(ContractError::generic(format!(
                "value not found in storage: key={}",
                String::from_utf8(prefixed_key).unwrap(),
            )))
    }

    pub fn set_prefixed(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let prefixed_key = self.prefixed_key(key);
        self.storage_mut().set(&prefixed_key, value.as_ref());
    }

    pub fn remove_prefixed(&mut self, key: &[u8]) {
        let prefixed_key = self.prefixed_key(key);
        self.storage_mut().remove(&prefixed_key);
    }

    pub fn host_height(&self) -> Height {
        Height::new(0, self.env.block.height)
    }
}

impl<'a, C: CustomQuery> KVStore for Context<'a, C> {
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.set_prefixed(key, value)
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get_prefixed(key).ok()
    }

    fn remove(&mut self, key: &[u8]) {
        self.remove_prefixed(key)
    }
}

impl<'a, C: CustomQuery> HostContext for Context<'a, C> {
    fn host_timestamp(&self) -> Time {
        Time::from_unix_timestamp_nanos(self.env.block.time.nanos() as u128).unwrap()
    }
}

impl<'a, C: CustomQuery> ClientReader for Context<'a, C> {
    fn client_exists(&self, _client_id: &ClientId) -> bool {
        unimplemented!();
    }

    fn client_type(&self, _client_id: &ClientId) -> Result<String, LightError> {
        unimplemented!();
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Any, LightError> {
        let value = self
            .get(CLIENT_STATE.as_bytes())
            .ok_or(LightError::client_state_not_found(client_id.clone()))?;
        let any_wasm_client_state = Any::decode(value.as_slice()).map_err(|e| {
            WasmLightClientSpecificError::NotAnyWasmClientState(e, client_id.clone())
        })?;
        let wasm_client_state: WasmClientState = any_wasm_client_state
            .try_into()
            .map_err(|e| WasmLightClientSpecificError::NotWasmClientState(e, client_id.clone()))?;
        let any_client_state = Any::decode(wasm_client_state.data.as_slice())
            .map_err(|e| WasmLightClientSpecificError::NotAnyClientState(e, client_id.clone()))?;
        Ok(any_client_state)
    }

    fn consensus_state(&self, client_id: &ClientId, height: &Height) -> Result<Any, LightError> {
        let path = format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}",
            height.revision_number(),
            height.revision_height(),
        );
        let value = self
            .get(path.as_bytes())
            .ok_or(LightError::consensus_state_not_found(
                client_id.clone(),
                *height,
            ))?;
        let any_wasm_consensus_state = Any::decode(value.as_slice()).map_err(|e| {
            WasmLightClientSpecificError::NotAnyWasmConsensusState(e, client_id.clone(), *height)
        })?;
        let wasm_consensus_state: WasmConsensusState =
            any_wasm_consensus_state.try_into().map_err(|e| {
                WasmLightClientSpecificError::NotWasmConsensusState(e, client_id.clone(), *height)
            })?;
        let any_consensus_state =
            Any::decode(wasm_consensus_state.data.as_slice()).map_err(|e| {
                WasmLightClientSpecificError::NotAnyConsensusState(e, client_id.clone(), *height)
            })?;
        Ok(any_consensus_state)
    }
}

impl<'a, C: CustomQuery> HostClientReader for Context<'a, C> {}

pub trait ExecutionContext: KVStore {
    type Error;

    fn store_client_state(
        &mut self,
        latest_height: Height,
        any_client_state: Any,
    ) -> Result<(), Self::Error>;

    fn store_consensus_state(
        &mut self,
        height: Height,
        any_consensus_state: Any,
    ) -> Result<(), Self::Error>;

    fn delete_consensus_state(&mut self, height: Height) -> Result<(), Self::Error>;

    fn store_update_meta(
        &mut self,
        height: Height,
        host_timestamp: Time,
        host_height: Height,
    ) -> Result<(), Self::Error>;

    fn delete_update_meta(&mut self, height: Height) -> Result<(), Self::Error>;
}

impl<'a, C: CustomQuery> ExecutionContext for Context<'a, C> {
    type Error = ContractError;

    fn store_client_state(
        &mut self,
        latest_height: Height,
        any_client_state: Any,
    ) -> Result<(), Self::Error> {
        let prefixed_key = self.prefixed_key(CLIENT_STATE);

        let wasm_client_state = WasmClientState {
            checksum: self.obtain_checksum()?.into(),
            latest_height: Some(latest_height.into()),
            data: any_client_state.encode_to_vec(),
        };

        let any_wasm_client_state = Any::from(wasm_client_state);

        self.set(prefixed_key, any_wasm_client_state.encode_to_vec());

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        height: Height,
        any_consensus_state: Any,
    ) -> Result<(), Self::Error> {
        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}",
            height.revision_number(),
            height.revision_height(),
        ));

        let wasm_consensus_state = WasmConsensusState {
            data: any_consensus_state.encode_to_vec(),
        };

        let any_wasm_consensus_state = Any::from(wasm_consensus_state);

        self.set(prefixed_key, any_wasm_consensus_state.encode_to_vec());

        Ok(())
    }

    fn delete_consensus_state(&mut self, height: Height) -> Result<(), Self::Error> {
        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}",
            height.revision_number(),
            height.revision_height(),
        ));

        self.remove(&prefixed_key);

        Ok(())
    }

    fn store_update_meta(
        &mut self,
        height: Height,
        host_timestamp: Time,
        host_height: Height,
    ) -> Result<(), Self::Error> {
        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_TIME}",
            height.revision_number(),
            height.revision_height(),
        ));
        let time_vec = u64::try_from(host_timestamp.as_unix_timestamp_nanos())
            .unwrap()
            .to_be_bytes();
        self.set(prefixed_key, time_vec.into());

        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_HEIGHT}",
            height.revision_number(),
            height.revision_height(),
        ));
        let revision_height_vec = host_height.revision_height().to_be_bytes();
        self.set(prefixed_key, revision_height_vec.into());

        Ok(())
    }

    fn delete_update_meta(&mut self, height: Height) -> Result<(), Self::Error> {
        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_TIME}",
            height.revision_number(),
            height.revision_height(),
        ));
        self.remove(&prefixed_key);

        let prefixed_key = self.prefixed_key(format!(
            "{CONSENSUS_STATE_PREFIX}/{}-{}/{PROCESSED_HEIGHT}",
            height.revision_number(),
            height.revision_height(),
        ));
        self.remove(&prefixed_key);

        Ok(())
    }
}
