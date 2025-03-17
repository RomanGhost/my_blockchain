use crate::coin::node::blockchain::transaction::SerializedTransaction;

pub enum TransactionMessage{
    AddTransaction(SerializedTransaction),
    GetTransaction(),
    TransactionVec(Vec<SerializedTransaction>),
}