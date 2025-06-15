use std::fmt::Debug;
use std::num::{ParseFloatError, ParseIntError};

use serde_json::{self, Value};

use crate::api::v1::structs::{
    ABIResponse, GetBlockResponse, GetTransactionStatusResponse,
    SendTransaction2Request
};
use crate::chain::checksum::{Checksum160, Checksum256};
use crate::{
    api::{
        client::Provider,
        v1::structs::{
            AccountObject, GetInfoResponse, GetTableRowsParams,
            GetTableRowsResponse, SendTransaction2Options, SendTransaction2Response,
            SendTransactionResponse,
            TableIndexType,
        },
    },
    chain::{
        name::Name,
        transaction::{CompressionType, PackedTransaction, SignedTransaction},
    },
    name,
    serializer::{Decoder, Packer},
};
use hex::decode;

use super::structs::{ChainAPIError, ChainResult, NodeosErrorDetail, NodeosErrorEnvelope};

#[derive(Debug, Default, Clone)]
pub struct ChainAPI<T: Provider> {
    provider: T,
}

impl<T: Provider> ChainAPI<T> {
    pub fn new(provider: T) -> Self {
        ChainAPI { provider }
    }

    pub async fn get_account(
        &self,
        account_name: String,
    ) -> ChainResult<AccountObject> {
        let payload = serde_json::json!({ "account_name": account_name });

        let response = self
            .provider
            .post(
                String::from("/v1/chain/get_account"),
                Some(payload.to_string()),
            )
            .await
            .map_err(ChainAPIError::from)?;

        serde_json::from_str::<AccountObject>(&response)
            .map_err(ChainAPIError::from)
    }

    pub async fn get_abi(
        &self,
        account_name: String,
    ) -> Result<ABIResponse, ChainAPIError> {
        let payload = serde_json::json!({
            "account_name": account_name,
        });

        let response = self
            .provider
            .post(
                String::from("/v1/chain/get_abi"),
                Some(payload.to_string()),
            )
            .await
            .map_err(ChainAPIError::Network)?;

        serde_json::from_str::<ABIResponse>(&response)
            .map_err(ChainAPIError::from)
    }

    pub async fn get_block(
        &self,
        block_num_or_id: String,
    ) -> Result<GetBlockResponse, ChainAPIError> {
        let payload = serde_json::json!({
            "block_num_or_id": block_num_or_id,
        });

        let response = self
            .provider
            .post(
                String::from("/v1/chain/get_block"),
                Some(payload.to_string()),
            )
            .await
            .map_err(ChainAPIError::Network)?;

        serde_json::from_str::<GetBlockResponse>(&response)
            .map_err(ChainAPIError::from)
    }

    pub async fn get_info(&self) -> Result<GetInfoResponse, ChainAPIError> {
        let response = self.provider.get(String::from("/v1/chain/get_info")).await?;

        serde_json::from_str::<GetInfoResponse>(&response)
            .map_err(ChainAPIError::from)
    }

    /// send_transaction sends transaction to telos using /v1/chain/send_transaction
    /// and using ZLIB compression type.
    pub async fn send_transaction(
        &self,
        trx: SignedTransaction,
    ) -> Result<SendTransactionResponse, ChainAPIError> {
        let packed = PackedTransaction::from_signed(trx, CompressionType::ZLIB)
            .map_err(|e| ChainAPIError::Parse(format!("pack tx: {e}")))?;

        let body = packed.to_json();
        let result = self
            .provider
            .post(
                String::from("/v1/chain/send_transaction"),
                Some(body.to_string()),
            )
            .await
            .map_err(ChainAPIError::from)?;

        match serde_json::from_str::<SendTransactionResponse>(&result) {
            Ok(ok) => Ok(ok),

            Err(_) => {
                // attempt to parse the standard eosio `error` envelope
                let err = serde_json::from_str::<NodeosErrorEnvelope>(&result)
                    .map_err(ChainAPIError::from)?;

                Err(ChainAPIError::Nodeos {
                    code:    err.error.code,
                    name:    err.error.name,
                    what:    err.error.what,
                    details: err.error.details.unwrap_or(
                        vec![NodeosErrorDetail{message: "unknown error".to_string()}]
                    ).first().map(|d| d.message.clone()),
                })
            }
        }
    }

    /// send_transaction2 sends transaction to telos using /v1/chain/send_transaction2
    /// which enables retry in case of transaction failure using ZLIB compression type.
    pub async fn send_transaction2(
        &self,
        trx: SignedTransaction,
        options: Option<SendTransaction2Options>,
    ) -> Result<SendTransaction2Response, ChainAPIError> {
        let packed = PackedTransaction::from_signed(trx, CompressionType::ZLIB)
            .map_err(|e| ChainAPIError::Parse(format!("pack tx: {e}")))?;

        let req = SendTransaction2Request::build(packed, options);
        let body = serde_json::to_string(&req)
            .map_err(|e| ChainAPIError::Parse(format!("serialize body: {e}")))?;

        let result = self
            .provider
            .post(
                String::from("/v1/chain/send_transaction2"),
                Some(body),
            )
            .await
            .map_err(ChainAPIError::from)?;

        match serde_json::from_str::<SendTransaction2Response>(&result) {
            Ok(ok) => Ok(ok),

            Err(_) => {
                // attempt to parse the standard eosio `error` envelope
                let err = serde_json::from_str::<NodeosErrorEnvelope>(&result)
                    .map_err(ChainAPIError::from)?;

                Err(ChainAPIError::Nodeos {
                    code:    err.error.code,
                    name:    err.error.name,
                    what:    err.error.what,
                    details: err.error.details.unwrap_or(
                        vec![NodeosErrorDetail{message: "unknown error".to_string()}]
                    ).first().map(|d| d.message.clone()),
                })
            }
        }
    }

    pub async fn get_transaction_status(
        &self,
        trx_id: Checksum256,
    ) -> Result<GetTransactionStatusResponse, ChainAPIError> {
        let payload = serde_json::json!({ "id": trx_id.to_string() });

        let result = self
            .provider
            .post(
                String::from("/v1/chain/get_transaction_status"),
                Some(payload.to_string()),
            )
            .await
            .map_err(ChainAPIError::from)?;

        serde_json::from_str::<GetTransactionStatusResponse>(&result) 
            .map_err(ChainAPIError::from)
    }

    pub async fn get_table_rows<P: Packer + Default>(
        &self,
        params: GetTableRowsParams,
    ) -> Result<GetTableRowsResponse<P>, ChainAPIError> {
        let response = self
            .provider
            .post(
                String::from("/v1/chain/get_table_rows"),
                Some(params.to_json()),
            )
            .await
            .map_err(ChainAPIError::Network)?;

        let json: Value = serde_json::from_str(&response)?;

        let obj = json
            .as_object()
            .ok_or_else(|| ChainAPIError::Parse("response is not object".into()))?;

        let more = obj
            .get("more")
            .and_then(Value::as_bool)
            .ok_or_else(|| ChainAPIError::Parse("'more' missing or not bool".into()))?;

        let next_key_str = obj
            .get("next_key")
            .and_then(Value::as_str)
            .ok_or_else(|| ChainAPIError::Parse("'next_key' missing".into()))?
            .to_string();

        let rows_arr = obj
            .get("rows")
            .and_then(Value::as_array)
            .ok_or_else(|| ChainAPIError::Parse("'rows' missing or not array".into()))?
            .clone();

        let mut rows = Vec::with_capacity(rows_arr.len());
        for encoded in rows_arr {
            let row_hex = encoded
                .as_str()
                .ok_or_else(|| ChainAPIError::Parse("row not string".into()))?;
            let row_bytes = decode(row_hex)
                .map_err(|e| ChainAPIError::Parse(e.to_string()))?;
            let mut decoder = Decoder::new(&row_bytes);
            let mut row = P::default();
            decoder
                .unpack(&mut row)
                .map_err(|e| ChainAPIError::Parse(e.to_string()))?;
            rows.push(row);
        }

        let mut next_key = None;
        if !next_key_str.is_empty() {
            next_key = match params.lower_bound {
                Some(TableIndexType::NAME(_)) => {
                    Some(TableIndexType::NAME(name!(&next_key_str)))
                }
                Some(TableIndexType::UINT64(_)) => Some(TableIndexType::UINT64(
                    next_key_str
                        .parse()
                        .map_err(|e: ParseIntError| ChainAPIError::Parse(e.to_string()))?,
                )),
                Some(TableIndexType::UINT128(_)) => Some(TableIndexType::UINT128(
                    next_key_str
                        .parse()
                        .map_err(|e: ParseIntError| ChainAPIError::Parse(e.to_string()))?,
                )),
                Some(TableIndexType::CHECKSUM160(_)) => Some(
                    TableIndexType::CHECKSUM160(
                        Checksum160::try_from(
                            decode(&next_key_str)
                                .map_err(|e| ChainAPIError::Parse(e.to_string()))?
                                .as_slice(),
                        )
                        .map_err(|e| ChainAPIError::Parse(format!("bad checksum160: {e}")))?,
                    ),
                ),
                Some(TableIndexType::CHECKSUM256(_)) => Some(
                    TableIndexType::CHECKSUM256(
                        Checksum256::try_from(
                            decode(&next_key_str)
                                .map_err(|e| ChainAPIError::Parse(e.to_string()))?
                                .as_slice(),
                        )
                        .map_err(|e| ChainAPIError::Parse(format!("bad checksum256: {e}")))?,
                    ),
                ),
                Some(TableIndexType::FLOAT64(_)) => Some(TableIndexType::FLOAT64(
                    next_key_str
                        .parse()
                        .map_err(|e: ParseFloatError| ChainAPIError::Parse(e.to_string()))?,
                )),
                None => None,
            };
        }

        Ok(GetTableRowsResponse {
            rows,
            more,
            ram_payers: None,
            next_key,
        })
    }
}
