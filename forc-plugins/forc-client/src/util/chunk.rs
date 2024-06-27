//! FuelVM has a specific limit on contract-size, but some contracts are bigger
//! than the limit and needs to be split into chunks before deployment.
//!
//! If the contract needs to be "chunked":
//!  1. Find out number of chunks needed.
//!  2. Split bytecode into predetermined number of chunks.
//!  3. Deploy each chunk as a seperate contract and collect their contract ids.
//!  4. Generate a "loader" contract.
use fuel_tx::ContractId;
use fuels_accounts::provider::Provider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractChunk {
    id: usize,
    size: usize,
    bytecode: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeployedContractChunk {
    chunk: ContractChunk,
    contract_id: ContractId,
}

impl DeployedContractChunk {
    pub fn chunk(&self) -> &ContractChunk {
        &self.chunk
    }
    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
}

impl ContractChunk {
    pub fn new(id: usize, size: usize, bytecode: Vec<u8>) -> Self {
        Self { id, size, bytecode }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    pub async fn deploy(provider: &Provider) -> anyhow::Result<DeployedContractChunk> {
        todo!()
    }
}

/// Split bytecode into chunks of a specified maximum size.
pub fn split_into_chunks(bytecode: Vec<u8>, chunk_size: usize) -> Vec<ContractChunk> {
    let mut chunks = Vec::new();
    let mut id = 0;

    for chunk in bytecode.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let size = chunk.len();
        let contract_chunk = ContractChunk::new(id, size, chunk);
        chunks.push(contract_chunk);
        id += 1;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_chunks_exact_division() {
        let bytecode = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let chunk_size = 4;
        let chunks = split_into_chunks(bytecode.clone(), chunk_size);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], ContractChunk::new(0, 4, vec![1, 2, 3, 4]));
        assert_eq!(chunks[1], ContractChunk::new(1, 4, vec![5, 6, 7, 8]));
    }

    #[test]
    fn test_split_into_chunks_with_remainder() {
        let bytecode = vec![1, 2, 3, 4, 5, 6, 7];
        let chunk_size = 4;
        let chunks = split_into_chunks(bytecode.clone(), chunk_size);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], ContractChunk::new(0, 4, vec![1, 2, 3, 4]));
        assert_eq!(chunks[1], ContractChunk::new(1, 3, vec![5, 6, 7]));
    }

    #[test]
    fn test_split_into_chunks_empty_bytecode() {
        let bytecode = vec![];
        let chunk_size = 4;
        let chunks = split_into_chunks(bytecode.clone(), chunk_size);

        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_split_into_chunks_smaller_than_chunk_size() {
        let bytecode = vec![1, 2, 3];
        let chunk_size = 4;
        let chunks = split_into_chunks(bytecode.clone(), chunk_size);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], ContractChunk::new(0, 3, vec![1, 2, 3]));
    }
}
