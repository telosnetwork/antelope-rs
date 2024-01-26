use crate::api::v1::structs::{ActionReceipt, ActionTrace, AuthSequence, AccountRamDelta, EncodingError};
use crate::chain::action::{Action, PermissionLevel};
use crate::chain::name::Name;
use crate::name;
use crate::serializer::formatter::JSONObject;
use serde_json::Value;

pub fn parse_action_traces(action_traces_json: Value) -> Result<Vec<ActionTrace>, EncodingError> {
    let action_traces_array = action_traces_json
        .as_array()
        .ok_or(EncodingError::new("Invalid action traces array".into()))?;
    let mut result = Vec::new();

    for action_trace_json in action_traces_array {
        let json = JSONObject::new(action_trace_json.clone());
        let action_trace = parse_action_trace(json)?;
        result.push(action_trace)
    }
    Ok(result)
}

pub fn parse_action_trace(json: JSONObject) -> Result<ActionTrace, EncodingError> {
    let action_ordinal = json.get_u32("action_ordinal")?;
    let creator_action_ordinal = json.get_u32("creator_action_ordinal")?;
    let closest_unnotified_ancestor_action_ordinal =
        json.get_u32("closest_unnotified_ancestor_action_ordinal")?;

    let act_json = json.get_value("act")?;
    let act = parse_action(Some(act_json))?;

    let receipt_json = json.get_value("receipt")?;
    let receipt = parse_action_receipt(receipt_json)?;

    let receiver = json.get_string("receiver")?;

    let account_ram_deltas_json = json.get_value("account_ram_deltas").unwrap_or(Value::Array(vec![]));
    let account_ram_deltas = parse_account_ram_deltas(account_ram_deltas_json)?;

    let elapsed = json.get_u64("elapsed")?;
    let context_free = json.get_bool("context_free")?;
    let console = json.get_string("console")?;
    let trx_id = json.get_string("trx_id")?;
    let block_num = json.get_u64("block_num")?;
    let block_time = json.get_string("block_time")?;
    let producer_block_id = json.get_optional_string("producer_block_id")?;
    let except = json.get_optional_string("except")?;
    let error_code = json.get_optional_u32("error_code")?;
    let return_value_hex_data = json.get_string("return_value_hex_data")?;

    Ok(ActionTrace {
        action_ordinal,
        creator_action_ordinal,
        closest_unnotified_ancestor_action_ordinal,
        receipt,
        receiver: name!(receiver.as_str()),
        act,
        elapsed,
        context_free,
        console,
        trx_id,
        block_num,
        block_time,
        producer_block_id,
        account_ram_deltas,
        except,
        error_code,
        return_value_hex_data,
    })
}

fn parse_action_receipt(json: Value) -> Result<ActionReceipt, EncodingError> {
    let json_obj = JSONObject::new(json.clone());
    let receiver = json_obj.get_string("receiver")?;
    let act_digest = json_obj.get_string("act_digest")?;
    let global_sequence = json_obj.get_u64("global_sequence")?;
    let recv_sequence = json_obj.get_u64("recv_sequence")?;
    let auth_sequence_json = json.get("auth_sequence").ok_or(EncodingError::new(
        "Missing 'auth_sequence' in action receipt".into(),
    ))?;
    let auth_sequence = parse_auth_sequence(Some(auth_sequence_json.clone()))?;
    let code_sequence = json_obj.get_u64("code_sequence")?;
    let abi_sequence = json_obj.get_u64("abi_sequence")?;

    Ok(ActionReceipt {
        receiver: name!(receiver.as_str()),
        act_digest,
        global_sequence,
        recv_sequence,
        auth_sequence,
        code_sequence,
        abi_sequence,
    })
}

fn parse_action(json: Option<Value>) -> Result<Action, EncodingError> {
    if let Some(json) = json {
        let json_obj = JSONObject::new(json);
        let account = json_obj.get_string("account")?;
        let name = json_obj.get_string("name")?;
        let authorization_json = json_obj.get_value("authorization")?;
        let authorization = parse_authorization(authorization_json)?;

        let data_obj = json_obj.get_value("data")?;

        let data_str = serde_json::to_string(&data_obj)
            .map_err(|_| EncodingError::new("Failed to serialize 'data' object".into()))?;

        let data = data_str.into_bytes();

        Ok(Action {
            account: name!(account.as_str()),
            name: name!(name.as_str()),
            authorization,
            data,
        })
    } else {
        Err(EncodingError::new("Missing 'act' in action trace".into()))
    }
}

fn parse_auth_sequence(json: Option<Value>) -> Result<Vec<AuthSequence>, EncodingError> {
    if let Some(json) = json {
        let auth_sequence_array = json
            .as_array()
            .ok_or(EncodingError::new("Invalid auth sequence array".into()))?;
        let mut result = Vec::new();
        for auth_sequence_json in auth_sequence_array {
            let auth_sequence_vec = auth_sequence_json
                .as_array()
                .ok_or(EncodingError::new("Invalid auth sequence".into()))?;
            if auth_sequence_vec.len() != 2 {
                return Err(EncodingError::new("Invalid auth sequence".into()));
            }
            let account = auth_sequence_vec[0].as_str().ok_or(EncodingError::new(
                "Invalid account in auth sequence".into(),
            ))?;
            let sequence = auth_sequence_vec[1].as_u64().ok_or(EncodingError::new(
                "Invalid sequence in auth sequence".into(),
            ))?;
            result.push(AuthSequence {
                account: name!(account),
                sequence,
            });
        }
        Ok(result)
    } else {
        Err(EncodingError::new(
            "Missing 'auth_sequence' in action receipt".into(),
        ))
    }
}

fn parse_authorization(json: Value) -> Result<Vec<PermissionLevel>, EncodingError> {
    let authorization_array = json
        .as_array()
        .ok_or(EncodingError::new("Invalid authorization array".into()))?;
    let mut result = Vec::new();

    for authorization_json in authorization_array {
        let authorization_obj = JSONObject::new(authorization_json.clone());
        let actor = authorization_obj.get_string("actor")?;
        let permission = authorization_obj.get_string("permission")?;
        result.push(PermissionLevel {
            actor: name!(actor.as_str()),
            permission: name!(permission.as_str()),
        });
    }

    Ok(result)
}

fn parse_account_ram_deltas(json: Value) -> Result<Vec<AccountRamDelta>, EncodingError> {
    let deltas_array = json
        .as_array()
        .ok_or(EncodingError::new("Invalid account_ram_deltas array".into()))?;

    let mut deltas = Vec::new();
    for delta_json in deltas_array {
        let delta_obj = JSONObject::new(delta_json.clone());
        let delta = parse_account_ram_delta(delta_obj)?;
        deltas.push(delta);
    }

    Ok(deltas)
}

pub fn parse_account_ram_delta(json: JSONObject) -> Result<AccountRamDelta, EncodingError> {
    let account = json.get_string("account")?;
    let delta = json.get_string("delta")?
        .parse::<i64>()
        .map_err(|_| EncodingError::new("Invalid delta value".into()))?;

    Ok(AccountRamDelta {
        account: name!(account.as_str()),
        delta,
    })
}
