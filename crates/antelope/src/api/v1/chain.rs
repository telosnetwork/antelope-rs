use std::fmt::Debug;

use serde_json::{self, Value};

use crate::api::v1::structs::{EncodingError, ServerError};
use crate::{
    api::{
        client::Provider,
        v1::structs::{
            AccountObject, ClientError, ErrorResponse, GetInfoResponse, GetTableRowsParams,
            GetTableRowsResponse, SendTransactionResponse, SendTransactionResponseError,
            TableIndexType,
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

    // pub async fn get_abi(&self) -> Result<

    pub async fn get_account(
        &self,
        account_name: String,
    ) -> Result<AccountObject, ClientError<()>> {
        let payload = serde_json::json!({ "account_name": account_name });

        let result = self
            .provider
            .post(
                String::from("/v1/chain/get_account"),
                Some(payload.to_string()),
            )
            .await;

        match result {
            Ok(response) => serde_json::from_str::<AccountObject>(&response)
                .map_err(|_| ClientError::encoding("Failed to parse JSON".into())),
            Err(_) => Err(ClientError::encoding("Request failed".into())),
        }
    }

    pub async fn get_info(&self) -> Result<GetInfoResponse, ClientError<()>> {
        let result = self.provider.get(String::from("/v1/chain/get_info")).await;

        match result {
            Ok(response) => serde_json::from_str::<GetInfoResponse>(&response)
                .map_err(|_| ClientError::encoding("Failed to parse JSON".into())),
            Err(_) => Err(ClientError::encoding("Request failed".into())),
        }
    }

    pub async fn send_transaction(
        &self,
        trx: SignedTransaction,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let packed = PackedTransaction::from_signed(trx, CompressionType::ZLIB)
            .map_err(|_| ClientError::encoding("Failed to pack transaction".into()))?;

        let trx_json = packed.to_json();
        let result = self
            .provider
            .post(String::from("/v1/chain/send_transaction"), Some(trx_json))
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
                            message: format!("Failed to parse response: {}", e),
                        }))
                    }
                }
            }
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

        let json: Value = serde_json::from_str(result.await.unwrap().as_str()).unwrap();
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

        let next_key = TableIndexType::NAME(name!(next_key_str.as_str()));

        Ok(GetTableRowsResponse {
            rows,
            more,
            ram_payers: None,
            next_key: Some(next_key),
        })
    }
}
