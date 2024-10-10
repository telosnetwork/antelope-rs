pub mod structs;

use crate::api::client::{APIClient, Provider};
use crate::api::system::structs::{
    CreateAccountParams, DelegateBandwidthAction, NewAccountAction, SetAbiAction, SetCodeAction,
    TransferAction,
};
use crate::api::v1::structs::{ClientError, SendTransactionResponse, SendTransactionResponseError};
use crate::chain::abi::ABI;
use crate::chain::action::{Action, PermissionLevel};
use crate::chain::binary_extension::BinaryExtension;
use crate::chain::name::Name;
use crate::chain::private_key::PrivateKey;
use crate::name;
use crate::serializer::Encoder;
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::info;

#[derive(Debug, Default, Clone)]
pub struct SystemAPI<T: Provider> {
    api_client: APIClient<T>,
}

impl<T: Provider> SystemAPI<T> {
    pub fn new(api_client: APIClient<T>) -> Self {
        SystemAPI { api_client }
    }

    pub async fn create_account(
        &self,
        create_params: CreateAccountParams,
        creator_private_key: PrivateKey,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let CreateAccountParams {
            name,
            creator,
            owner,
            active,
            ram_bytes,
            stake_net,
            stake_cpu,
            transfer,
        } = create_params;
        let new_account_action = Action::new(
            name!("eosio"),
            name!("newaccount"),
            PermissionLevel::new(creator, name!("active")),
            NewAccountAction {
                creator,
                name,
                owner,
                active,
            },
        );
        let buy_ram_action = Action::new(
            name!("eosio"),
            name!("buyrambytes"),
            PermissionLevel::new(creator, name!("active")),
            structs::BuyRamBytesAction {
                payer: creator,
                receiver: name,
                bytes: ram_bytes,
            },
        );
        let delegate_bw_action = Action::new(
            name!("eosio"),
            name!("delegatebw"),
            PermissionLevel::new(creator, name!("active")),
            DelegateBandwidthAction {
                from: creator,
                receiver: name,
                stake_net_quantity: stake_net,
                stake_cpu_quantity: stake_cpu,
                transfer,
            },
        );
        let actions = vec![new_account_action, buy_ram_action, delegate_bw_action];
        self.api_client.transact(actions, creator_private_key).await
    }

    pub async fn transfer(
        &self,
        transfer_action: TransferAction,
        sender_private_key: PrivateKey,
        token_contract: Option<Name>,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        self.api_client
            .transact(
                vec![Action::new(
                    token_contract.unwrap_or(name!("eosio.token")),
                    name!("transfer"),
                    PermissionLevel::new(transfer_action.from, name!("active")),
                    transfer_action,
                )],
                sender_private_key,
            )
            .await
    }

    pub async fn set_contract_from_files(
        &self,
        account: Name,
        wasm_path: &str,
        abi_path: &str,
        memo: Option<String>,
        private_key: PrivateKey,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let wasm = std::fs::read(Path::new(wasm_path)).unwrap();
        let abi_json_bytes = std::fs::read(Path::new(abi_path)).unwrap();
        let abi: ABI = serde_json::from_slice(&abi_json_bytes).unwrap();
        let abi_bytes = Encoder::pack(&abi);
        self.set_contract(account, wasm, abi_bytes, memo, private_key)
            .await
    }

    pub async fn set_contract(
        &self,
        account: Name,
        wasm: Vec<u8>,
        abi: Vec<u8>,
        memo: Option<String>,
        private_key: PrivateKey,
    ) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let mut hasher = Sha256::new();
        hasher.update(&wasm);
        let wasm_hash = hasher.finalize();
        info!(
            "Setting contract for account: {:?}, with hash: {:?}",
            account.as_string(),
            wasm_hash
        );
        self.api_client
            .transact(
                vec![
                    Action::new(
                        name!("eosio"),
                        name!("setcode"),
                        PermissionLevel::new(account, name!("active")),
                        SetCodeAction {
                            vmtype: 0,
                            vmversion: 0,
                            account,
                            code: wasm,
                            memo: BinaryExtension::new(memo.clone()),
                        },
                    ),
                    Action::new(
                        name!("eosio"),
                        name!("setabi"),
                        PermissionLevel::new(account, name!("active")),
                        SetAbiAction {
                            account,
                            abi,
                            memo: BinaryExtension::new(memo),
                        },
                    ),
                ],
                private_key,
            )
            .await
    }
}
