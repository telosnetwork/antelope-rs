use std::fmt::Debug;

use serde_json::{self, Value};

use crate::api::v1::structs::{
    ABIResponse, EncodingError, GetBlockResponse, GetTransactionStatusResponse,
    SendTransaction2Request, ServerError,
};
use crate::chain::checksum::{Checksum160, Checksum256};
use crate::{
    api::{
        client::Provider,
        v1::structs::{
            AccountObject, ClientError, ErrorResponse, ErrorResponse2, GetInfoResponse,
            GetTableRowsParams, GetTableRowsResponse, SendTransaction2Options,
            SendTransaction2Response, SendTransactionResponse, SendTransactionResponse2Error,
            SendTransactionResponseError, TableIndexType,
        },
    },
    chain::{
        name::Name,
        transaction::{CompressionType, PackedTransaction, SignedTransaction},
        Decoder, Packer,
    },
    name,
    serializer::formatter::{JSONObject, ValueTo},
    util::hex_to_bytes,
};

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
    ) -> Result<AccountObject, ClientError<ErrorResponse>> {
        let payload = serde_json::json!({ "account_name": account_name });

        let result = self
            .provider
            .post(
                String::from("/v1/chain/get_account"),
                Some(payload.to_string()),
            )
            .await;

        match result {
            Ok(response) => {
                match serde_json::from_str::<AccountObject>(&response) {
                    Ok(account_object) => Ok(account_object),
                    Err(_) => {
                        // Attempt to parse the error response
                        match serde_json::from_str::<ErrorResponse>(&response) {
                            Ok(error_response) => Err(ClientError::SERVER(ServerError {
                                error: error_response,
                            })),
                            Err(_) => Err(ClientError::ENCODING(EncodingError {
                                message: "Failed to parse JSON".into(),
                            })),
                        }
                    }
                }
            }
            Err(msg) => Err(ClientError::NETWORK(msg)),
        }
    }

    pub async fn get_abi(
        &self,
        account_name: String,
    ) -> Result<ABIResponse, ClientError<ErrorResponse>> {
        let payload = serde_json::json!({
            "account_name": account_name,
        });

        let result = self
            .provider
            .post(String::from("/v1/chain/get_abi"), Some(payload.to_string()))
            .await;

        match result {
            Ok(response) => {
                match serde_json::from_str::<ABIResponse>(&response) {
                    Ok(abi_response) => Ok(abi_response),
                    Err(_) => {
                        // Attempt to parse the error response
                        match serde_json::from_str::<ErrorResponse>(&response) {
                            Ok(error_response) => Err(ClientError::SERVER(ServerError {
                                error: error_response,
                            })),
                            Err(_) => Err(ClientError::ENCODING(EncodingError {
                                message: "Failed to parse JSON".into(),
                            })),
                        }
                    }
                }
            }
            Err(msg) => Err(ClientError::NETWORK(msg)),
        }
    }

    pub async fn get_block(
        &self,
        block_num_or_id: String,
    ) -> Result<GetBlockResponse, ClientError<ErrorResponse>> {
        let payload = serde_json::json!({
            "block_num_or_id": block_num_or_id,
        });

        let result = self
            .provider
            .post(
                String::from("/v1/chain/get_block"),
                Some(payload.to_string()),
            )
            .await;

        match result {
            Ok(response) => {
                match serde_json::from_str::<GetBlockResponse>(&response) {
                    Ok(block_response) => Ok(block_response),
                    Err(_serr) => {
                        // Attempt to parse the error response
                        match serde_json::from_str::<ErrorResponse>(&response) {
                            Ok(error_response) => Err(ClientError::SERVER(ServerError {
                                error: error_response,
                            })),
                            Err(_) => Err(ClientError::ENCODING(EncodingError {
                                message: "Failed to parse JSON".into(),
                            })),
                        }
                    }
                }
            }
            Err(msg) => Err(ClientError::NETWORK(msg)),
        }
    }

    pub async fn get_info(&self) -> Result<GetInfoResponse, ClientError<()>> {
        let result = self.provider.get(String::from("/v1/chain/get_info")).await;

        match result {
            Ok(response) => serde_json::from_str::<GetInfoResponse>(&response).map_err(|e| {
                let message = format!("Failed to parse JSON: {}", e);
                ClientError::encoding(message)
            }),
            Err(_) => Err(ClientError::encoding("Request failed".into())),
        }
    }

    /// send_transaction sends transaction to telos using /v1/chain/send_transaction
    /// and using ZLIB compression type.
    pub async fn send_transaction(
        &self,
        trx: SignedTransaction,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let packed = PackedTransaction::from_signed(trx, CompressionType::ZLIB)
            .map_err(|_| ClientError::encoding("Failed to pack transaction".into()))?;

        let trx_json = packed.to_json();
        let result = self
            .provider
            .post(
                String::from("/v1/chain/send_transaction"),
                Some(trx_json.to_string()),
            )
            .await
            .map_err(|_| ClientError::NETWORK("Failed to send transaction".into()))?;

        // Try to deserialize the successful response
        match serde_json::from_str::<SendTransactionResponse>(&result) {
            Ok(response) => Ok(response),
            Err(_) => {
                // Attempt to parse the error response
                match serde_json::from_str::<ErrorResponse>(&result) {
                    Ok(error_response) => {
                        // Create a ClientError::SERVER error with the nested error from the response
                        Err(ClientError::SERVER(ServerError {
                            error: error_response.error,
                        }))
                    }
                    Err(e) => {
                        // If parsing the error response also fails, consider it an encoding error
                        Err(ClientError::ENCODING(EncodingError {
                            message: format!(
                                "Failed to parse response: {} Raw response was: {}",
                                e, result
                            ),
                        }))
                    }
                }
            }
        }
    }

    /// send_transaction2 sends transaction to telos using /v1/chain/send_transaction2
    /// which enables retry in case of transaction failure using ZLIB compression type.
    pub async fn send_transaction2(
        &self,
        trx: SignedTransaction,
        options: Option<SendTransaction2Options>,
    ) -> Result<SendTransaction2Response, ClientError<SendTransactionResponse2Error>> {
        let packed_transaction = PackedTransaction::from_signed(trx, CompressionType::ZLIB)
            .map_err(|_| ClientError::encoding("Failed to pack transaction".into()))?;

        let request_body = SendTransaction2Request::build(packed_transaction, options);

        let request_body_str = serde_json::to_string(&request_body)
            .map_err(|_| ClientError::encoding("Failed to serialize request body".into()))?;

        // Send the request to the endpoint
        let result = self
            .provider
            .post(
                String::from("/v1/chain/send_transaction2"),
                Some(request_body_str),
            )
            .await
            .map_err(|_| ClientError::NETWORK("Failed to send transaction".into()))?;

        // tracing::warn!("Result of the send_transaction2: {result}");

        // Deserialize the response
        match serde_json::from_str::<SendTransaction2Response>(&result) {
            Ok(response) => match response.processed.except {
                Some(error) => Err(ClientError::SERVER(ServerError { error })),
                None => Ok(response),
            },
            Err(error) => {
                tracing::error!("Failed to deserialize send_transactions2 response: {error}");

                // Try to parse an error response
                match serde_json::from_str::<ErrorResponse2>(&result) {
                    Ok(error_response) => Err(ClientError::SERVER(ServerError {
                        error: error_response.error,
                    })),
                    Err(e) => Err(ClientError::ENCODING(EncodingError {
                        message: format!("Failed to parse response: {}", e),
                    })),
                }
            }
        }
    }

    pub async fn get_transaction_status(
        &self,
        trx_id: Checksum256,
    ) -> Result<GetTransactionStatusResponse, ClientError<ErrorResponse>> {
        let payload = serde_json::json!({
            "id": trx_id.as_string(),
        });

        let result = self
            .provider
            .post(
                String::from("/v1/chain/get_transaction_status"),
                Some(payload.to_string()),
            )
            .await;

        match result {
            Ok(response) => {
                match serde_json::from_str::<GetTransactionStatusResponse>(&response) {
                    Ok(status_response) => Ok(status_response),
                    Err(err) => {
                        // Attempt to parse the error response
                        match serde_json::from_str::<ErrorResponse>(&response) {
                            Ok(error_response) => Err(ClientError::SERVER(ServerError {
                                error: error_response,
                            })),
                            Err(_) => Err(ClientError::ENCODING(EncodingError {
                                message: err.to_string(),
                            })),
                        }
                    }
                }
            }
            Err(msg) => Err(ClientError::NETWORK(msg)),
        }
    }

    pub async fn get_table_rows<P: Packer + Default>(
        &self,
        params: GetTableRowsParams,
    ) -> Result<GetTableRowsResponse<P>, ClientError<()>> {
        let result = self.provider.post(
            String::from("/v1/chain/get_table_rows"),
            Some(params.to_json()),
        );

        let response = match result.await {
            Ok(response) => response,
            Err(_) => return Err(ClientError::NETWORK("Failed to get table rows".into())),
        };
        let json: Value = serde_json::from_str(response.as_str()).unwrap();
        let response_obj = JSONObject::new(json);
        let more = response_obj.get_bool("more")?;
        let next_key_str = response_obj.get_string("next_key")?;
        let rows_value = response_obj.get_vec("rows")?;
        let mut rows: Vec<P> = Vec::with_capacity(rows_value.len());
        for encoded_row in rows_value {
            let row_bytes_hex = &ValueTo::string(Some(encoded_row))?;
            let row_bytes = hex_to_bytes(row_bytes_hex);
            let mut decoder = Decoder::new(&row_bytes);
            let mut row = P::default();
            decoder.unpack(&mut row);
            rows.push(row);
        }

        let mut next_key = None;

        if !next_key_str.is_empty() {
            match params.lower_bound {
                Some(TableIndexType::NAME(_)) => {
                    next_key = Some(TableIndexType::NAME(name!(next_key_str.as_str())));
                }
                Some(TableIndexType::UINT64(_)) => {
                    next_key = Some(TableIndexType::UINT64(next_key_str.parse().unwrap()));
                }
                Some(TableIndexType::UINT128(_)) => {
                    next_key = Some(TableIndexType::UINT128(next_key_str.parse().unwrap()));
                }
                Some(TableIndexType::CHECKSUM160(_)) => {
                    next_key = Some(TableIndexType::CHECKSUM160(
                        Checksum160::from_bytes(hex_to_bytes(&next_key_str).as_slice()).unwrap(),
                    ));
                }
                Some(TableIndexType::CHECKSUM256(_)) => {
                    next_key = Some(TableIndexType::CHECKSUM256(
                        Checksum256::from_bytes(hex_to_bytes(&next_key_str).as_slice()).unwrap(),
                    ));
                }
                Some(TableIndexType::FLOAT64(_)) => {
                    next_key = Some(TableIndexType::FLOAT64(next_key_str.parse().unwrap()));
                }
                None => {}
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
