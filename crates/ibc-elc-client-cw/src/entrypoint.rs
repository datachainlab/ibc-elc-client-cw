use crate::context::{Context, ExecutionContext};
use crate::error::ContractError;
use crate::msg::*;
use crate::response::*;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use light_client::types::Any;
use light_client::{ClientReader, HostContext, LightClient, UpdateClientResult};
use prost::Message;

pub trait Entrypoint {
    type LightClient: LightClient;

    fn get_status_from_client_state(any_client_state: Any) -> Result<String, ContractError>;
    fn get_timestamp_from_consensus_state(any_consensus_state: Any) -> Result<u64, ContractError>;

    fn instantiate(
        lc: &Self::LightClient,
        deps: DepsMut<'_>,
        env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let mut ctx = Context::new_mut(deps, env);
        ctx.set_checksum(msg.checksum);

        let any_client_state = Any::decode(&mut msg.client_state.as_slice())?;
        let any_consensus_state = Any::decode(&mut msg.consensus_state.as_slice())?;

        let res = lc.create_client(&ctx, any_client_state.clone(), any_consensus_state.clone())?;

        ctx.store_client_state(res.height, any_client_state)?;
        ctx.store_consensus_state(res.height, any_consensus_state)?;
        ctx.store_update_meta(res.height, ctx.host_timestamp().into(), ctx.host_height())?;

        Ok(Response::default().set_data(to_json_binary(&ContractResult::success())?))
    }

    fn sudo(
        lc: &Self::LightClient,
        deps: DepsMut<'_>,
        env: Env,
        msg: SudoMsg,
    ) -> Result<Response, ContractError> {
        let mut ctx = Context::new_mut(deps, env);

        let result = match msg {
            SudoMsg::UpdateState(msg) => {
                let any_message = Any::decode(msg.client_message.as_slice())?;
                let res = match lc.update_client(&ctx, ctx.client_id().clone(), any_message)? {
                    UpdateClientResult::UpdateState(d) => d,
                    _ => panic!("unexpected non-UpdateState client message"),
                };

                ctx.store_client_state(res.height, res.new_any_client_state)?;
                ctx.store_consensus_state(res.height, res.new_any_consensus_state)?;
                ctx.store_update_meta(res.height, ctx.host_timestamp().into(), ctx.host_height())?;

                ContractResult::success().heights(vec![res.height])
            }
            SudoMsg::UpdateStateOnMisbehaviour(msg) => {
                let any_message = Any::decode(msg.client_message.as_slice())?;
                let res = match lc.update_client(&ctx, ctx.client_id().clone(), any_message)? {
                    UpdateClientResult::Misbehaviour(d) => d,
                    _ => panic!("unexpected non-Misbehaviour client message"),
                };

                let latest_height = lc.latest_height(&ctx, &ctx.client_id())?;
                ctx.store_client_state(latest_height, res.new_any_client_state)?;

                ContractResult::success()
            }
            SudoMsg::VerifyUpgradeAndUpdateState(_) => {
                return Err(ContractError::unsupported(
                    "VerifyUpgradeAndUpdateState is not supported",
                ));
            }
            SudoMsg::VerifyMembership(msg) => {
                let (prefix, path) = msg.merkle_path.prefix_and_path()?;
                let _ = lc.verify_membership(
                    &ctx,
                    ctx.client_id().clone(),
                    prefix,
                    path,
                    msg.value.into(),
                    msg.height.into(),
                    msg.proof.into(),
                )?;

                ContractResult::success()
            }
            SudoMsg::VerifyNonMembership(msg) => {
                let (prefix, path) = msg.merkle_path.prefix_and_path()?;
                let _ = lc.verify_non_membership(
                    &ctx,
                    ctx.client_id().clone(),
                    prefix,
                    path,
                    msg.height.into(),
                    msg.proof.into(),
                )?;

                ContractResult::success()
            }
            SudoMsg::MigrateClientStore(_) => {
                return Err(ContractError::unsupported(
                    "MigrateClientStore is not supported",
                ));
            }
        };

        Ok(Response::default().set_data(to_json_binary(&result)?))
    }

    fn query(
        lc: Self::LightClient,
        deps: Deps<'_>,
        env: Env,
        msg: QueryMsg,
    ) -> Result<Binary, ContractError> {
        let ctx = Context::new_ref(deps, env);

        let retval = match msg {
            QueryMsg::Status(StatusMsg {}) => {
                let any_client_state = ctx.client_state(ctx.client_id())?;
                let status = Self::get_status_from_client_state(any_client_state)?;
                to_json_binary(&StatusResponse { status })?
            }
            QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
                return Err(ContractError::unsupported(
                    "ExportMetadata is not supported",
                ));
            }
            QueryMsg::TimestampAtHeight(msg) => {
                let any_consensus_state =
                    ctx.consensus_state(ctx.client_id(), &msg.height.into())?;
                let timestamp = Self::get_timestamp_from_consensus_state(any_consensus_state)?;
                to_json_binary(&TimestampAtHeightResponse { timestamp })?
            }
            QueryMsg::VerifyClientMessage(msg) => {
                let any_message = Any::decode(msg.client_message.as_slice())?;
                lc.update_client(&ctx, ctx.client_id().clone(), any_message)?;
                to_json_binary(&VerifyClientMessageResponse {})?
            }
            QueryMsg::CheckForMisbehaviour(msg) => {
                let any_message = Any::decode(msg.client_message.as_slice())?;
                let res = lc.update_client(&ctx, ctx.client_id().clone(), any_message)?;
                let found_misbehaviour = match res {
                    UpdateClientResult::Misbehaviour(_) => true,
                    _ => false,
                };
                to_json_binary(&CheckForMisbehaviourResponse { found_misbehaviour })?
            }
        };
        Ok(retval)
    }
}
