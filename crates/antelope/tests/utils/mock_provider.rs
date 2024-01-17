use antelope::api::client::{HTTPMethod, Provider};
use antelope::api::v1::structs::GetInfoResponse;
use antelope::chain::action::{Action, PermissionLevel};
use antelope::chain::asset::Asset;
use antelope::chain::checksum::Checksum160;
use antelope::chain::name::Name;
use antelope::chain::private_key::PrivateKey;
use antelope::chain::transaction::{SignedTransaction, Transaction};
use antelope::chain::{Decoder, Encoder, Packer};
use antelope::name;
use antelope_client_macros::StructPacker;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct MockProvider {}

impl MockProvider {
    fn call(
        &self,
        method: HTTPMethod,
        path: String,
        body: Option<String>,
    ) -> Result<String, String> {
        let mut to_hash = method.to_string() + &path;
        if body.is_some() {
            to_hash += body.unwrap().as_str();
        }

        let filename = Checksum160::hash(to_hash.into_bytes()).to_string();
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/utils/mock_provider_data/");
        d.push(filename + ".json");
        Ok(fs::read_to_string(&d).unwrap())
    }
}

impl Debug for MockProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockProvider")
    }
}

impl Provider for MockProvider {
    fn post(&self, path: String, body: Option<String>) -> Result<String, String> {
        self.call(HTTPMethod::POST, path, body)
    }

    fn get(&self, path: String) -> Result<String, String> {
        self.call(HTTPMethod::GET, path, None)
    }
}

pub fn make_mock_transaction(info: &GetInfoResponse, asset_to_transfer: Asset) -> Transaction {
    let trx_header = info.get_transaction_header(90);

    #[derive(Clone, Eq, PartialEq, Default, StructPacker)]
    struct Transfer {
        from: Name,
        to: Name,
        quantity: Asset,
        memo: String,
    }

    let transfer_data = Transfer {
        from: name!("corecorecore"),
        to: name!("teamgreymass"),
        quantity: asset_to_transfer,
        memo: String::from("Testing antelope-rs"),
    };

    let action = Action::new_ex(
        name!("eosio.token"),
        name!("transfer"),
        vec![PermissionLevel::new(name!("corecorecore"), name!("active"))],
        &transfer_data,
    );

    Transaction {
        header: trx_header,
        context_free_actions: vec![],
        actions: vec![action],
        extension: vec![],
    }
}

pub fn sign_mock_transaction(trx: &Transaction, info: &GetInfoResponse) -> SignedTransaction {
    let private_key =
        PrivateKey::from_str("5JW71y3njNNVf9fiGaufq8Up5XiGk68jZ5tYhKpy69yyU9cr7n9", false).unwrap();
    let sign_data = trx.signing_data(&info.chain_id.data.to_vec());
    SignedTransaction {
        transaction: trx.clone(),
        signatures: vec![private_key.sign_message(&sign_data)],
        context_free_data: vec![],
    }
}
