use std::fs;
use std::path::PathBuf;
use antelope::api::client::{HTTPMethod, Provider};
use antelope::api::v1::structs::GetInfoResponse;
use antelope::chain::asset::Asset;
use antelope::chain::checksum::Checksum160;
use antelope::chain::name::Name;
use antelope::chain::private_key::PrivateKey;
use antelope::chain::transaction::{SignedTransaction, Transaction};
use antelope::chain::{Packer, Encoder, Decoder};
use antelope::chain::action::{Action, PermissionLevel};
use antelope::name;
use antelope_macros::StructPacker;

pub struct MockProvider {
}

impl Provider for MockProvider {
    fn call(&self, method: HTTPMethod, path: String, body: Option<String>) -> Result<String, String> {
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

pub fn make_mock_transaction(info: &GetInfoResponse) -> Transaction {
    let trx_header = info.get_transaction_header(90);

    #[derive(Clone, Eq, PartialEq, Default, StructPacker)]
    struct Transfer {
        from: Name,
        to: Name,
        quantity: Asset,
        memo: String
    }

    let transfer_data = Transfer {
        from: name!("corecorecore"),
        to: name!("teamgreymass"),
        quantity: Asset::from_string("0.0420 TLOS"),
        memo: String::from("Testing antelope-rs"),
    };

    let action = Action::new_ex(
        name!("eosio.token"),
        name!("transfer"),
        vec![PermissionLevel::new(name!("corecorecore"), name!("active"))],
        &transfer_data
    );

    Transaction {
        header: trx_header,
        context_free_actions: vec![],
        actions: vec![action],
        extension: vec![],
    }
}

pub fn sign_mock_transaction(trx: Transaction, info: &GetInfoResponse) -> SignedTransaction {
    let private_key = PrivateKey::from_str("5JW71y3njNNVf9fiGaufq8Up5XiGk68jZ5tYhKpy69yyU9cr7n9", false).unwrap();
    let sign_data = trx.signing_data(&info.chain_id.data.to_vec());
    SignedTransaction {
        transaction: trx,
        signatures: vec![private_key.sign_message(&sign_data)],
        context_free_data: vec![],
    }
}