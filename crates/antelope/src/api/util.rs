use crate::api::client::{APIClient, Provider};
use crate::api::v1::structs::{ClientError, SendTransactionResponse, SendTransactionResponseError};
use crate::chain::action::Action;
use crate::chain::private_key::PrivateKey;
use crate::chain::transaction::{SignedTransaction, Transaction};

pub async fn transact<T: Provider>(
    api_client: &APIClient<T>,
    actions: Vec<Action>,
    private_key: PrivateKey,
) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
    let info = api_client.v1_chain.get_info().await.unwrap();
    let trx_header = info.get_transaction_header(90);
    let trx = Transaction {
        header: trx_header,
        context_free_actions: vec![],
        actions,
        extension: vec![],
    };

    let sign_data = trx.signing_data(&info.chain_id.data);

    let signed = SignedTransaction {
        transaction: trx,
        signatures: vec![private_key.sign_message(&sign_data)],
        context_free_data: vec![],
    };

    api_client.v1_chain.send_transaction(signed).await
}
