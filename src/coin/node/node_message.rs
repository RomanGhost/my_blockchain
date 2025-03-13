use crate::coin::node::blockchain::block::Block;
use crate::coin::node::blockchain::transaction::SerializedTransaction;

pub enum BlockchainMessage{
    ForceBlockAdd(Block),
    BlockAdd(Block),
    Chain(Vec<Block>),
    ChainCheck(Vec<Block>),
}

pub enum TransactionMessage{
    AddTransaction(SerializedTransaction),
    GetTransaction(),
    TransactionVec(Vec<SerializedTransaction>),
}