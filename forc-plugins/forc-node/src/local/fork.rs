use anyhow::Context;
use forc_pkg::source::reg::block_on_any_runtime;
use fuel_core::{
    database::{database_description::on_chain::OnChain, Database},
    state::{
        data_source::DataSourceType, iterable_key_value_view::IterableKeyValueViewWrapper,
        key_value_view::KeyValueViewWrapper, ColumnType, IterableKeyValueView, KeyValueView,
        TransactableStorage,
    },
};
use fuel_core_client::client::FuelClient;
use fuel_core_storage::{
    self,
    column::Column,
    iter::{BoxedIter, IterDirection, IterableStore},
    kv_store::{KVItem, KeyItem, KeyValueInspect, Value, WriteOperation},
    transactional::{Changes, ReferenceBytesKey, StorageChanges},
    Result as StorageResult,
};
use fuel_core_types::{
    fuel_tx::ContractId,
    fuel_types::{BlockHeight, Bytes32},
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct ForkSettings {
    pub fork_url: String,
    pub fork_block_height: Option<BlockHeight>,
}

impl ForkSettings {
    pub fn new(fork_url: String, fork_block_height: Option<BlockHeight>) -> Self {
        Self {
            fork_url,
            fork_block_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForkClient {
    client: FuelClient,
    block_height: Option<BlockHeight>,
}

impl ForkClient {
    pub fn new(fork_url: String, block_height: Option<BlockHeight>) -> anyhow::Result<Self> {
        let client = FuelClient::new(&fork_url)
            .with_context(|| format!("failed to create FuelClient for {fork_url}"))?;
        Ok(Self {
            client,
            block_height,
        })
    }

    pub fn fetch_contract_bytecode_blocking(
        &self,
        contract_id: &ContractId,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let client = self.client.clone();
        let contract_id = *contract_id;
        match block_on_any_runtime(async move { client.contract(&contract_id).await }) {
            Ok(Some(contract)) => Ok(Some(contract.bytecode)),
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::debug!("Failed to fetch contract bytecode: {}", e);
                Ok(None)
            }
        }
    }

    pub fn fetch_contract_state_blocking(
        &self,
        contract_id: &ContractId,
        key: &Bytes32,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let client = self.client.clone();
        let block_height = self.block_height;
        let contract_id = *contract_id;
        let key = *key;
        match block_on_any_runtime(async move {
            client
                .contract_slots_values(&contract_id, block_height, vec![key])
                .await
        }) {
            Ok(slot_values) => {
                for (slot_key, slot_value) in slot_values {
                    if slot_key == key {
                        return Ok(Some(slot_value));
                    }
                }
                Ok(None)
            }
            Err(e) => {
                tracing::debug!("Failed to fetch contract state: {}", e);
                Ok(None)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForkingOnChainStorage {
    data_source: DataSourceType<OnChain>,
    fork_client: Arc<ForkClient>,
}

impl ForkingOnChainStorage {
    pub fn new(on_chain: Database<OnChain>, fork_client: Arc<ForkClient>) -> Self {
        let data_source = on_chain.inner_storage().data.clone();
        Self {
            data_source,
            fork_client,
        }
    }

    pub fn wrap_iterable_view(
        &self,
        view: IterableKeyValueView<ColumnType<OnChain>, BlockHeight>,
    ) -> IterableKeyValueView<ColumnType<OnChain>, BlockHeight> {
        let (storage, metadata) = view.into_inner();
        let forked_store =
            ForkedView::new(storage, self.data_source.clone(), self.fork_client.clone());
        let wrapped = IterableKeyValueViewWrapper::new(forked_store);
        IterableKeyValueView::from_storage_and_metadata(wrapped, metadata)
    }

    pub fn wrap_key_value_view(
        &self,
        view: KeyValueView<ColumnType<OnChain>, BlockHeight>,
    ) -> KeyValueView<ColumnType<OnChain>, BlockHeight> {
        let (storage, metadata) = view.into_inner();
        let forked_store =
            ForkedView::new(storage, self.data_source.clone(), self.fork_client.clone());
        let wrapped = KeyValueViewWrapper::new(forked_store);
        KeyValueView::from_storage_and_metadata(wrapped, metadata)
    }
}

impl IterableStore for ForkingOnChainStorage {
    fn iter_store(
        &self,
        column: Self::Column,
        prefix: Option<&[u8]>,
        start: Option<&[u8]>,
        direction: IterDirection,
    ) -> BoxedIter<KVItem> {
        self.data_source
            .iter_store(column, prefix, start, direction)
    }

    fn iter_store_keys(
        &self,
        column: Self::Column,
        prefix: Option<&[u8]>,
        start: Option<&[u8]>,
        direction: IterDirection,
    ) -> BoxedIter<KeyItem> {
        self.data_source
            .iter_store_keys(column, prefix, start, direction)
    }
}

impl KeyValueInspect for ForkingOnChainStorage {
    type Column = ColumnType<OnChain>;

    fn exists(&self, key: &[u8], column: Self::Column) -> StorageResult<bool> {
        self.data_source.exists(key, column)
    }

    fn size_of_value(&self, key: &[u8], column: Self::Column) -> StorageResult<Option<usize>> {
        self.data_source.size_of_value(key, column)
    }

    fn get(&self, key: &[u8], column: Self::Column) -> StorageResult<Option<Value>> {
        self.data_source.get(key, column)
    }

    fn read(
        &self,
        key: &[u8],
        column: Self::Column,
        offset: usize,
        buf: &mut [u8],
    ) -> StorageResult<bool> {
        self.data_source.read(key, column, offset, buf)
    }
}

impl TransactableStorage<BlockHeight> for ForkingOnChainStorage {
    fn commit_changes(
        &self,
        height: Option<BlockHeight>,
        changes: StorageChanges,
    ) -> StorageResult<()> {
        self.data_source.commit_changes(height, changes)
    }

    fn view_at_height(
        &self,
        height: &BlockHeight,
    ) -> StorageResult<KeyValueView<ColumnType<OnChain>, BlockHeight>> {
        let view = self.data_source.view_at_height(height)?;
        Ok(self.wrap_key_value_view(view))
    }

    fn latest_view(&self) -> StorageResult<IterableKeyValueView<ColumnType<OnChain>, BlockHeight>> {
        let view = self.data_source.latest_view()?;
        Ok(self.wrap_iterable_view(view))
    }

    fn rollback_block_to(&self, height: &BlockHeight) -> StorageResult<()> {
        self.data_source.rollback_block_to(height)
    }

    fn shutdown(&self) {
        self.data_source.shutdown();
    }
}

struct ForkedView<V> {
    inner: V,
    storage: DataSourceType<OnChain>,
    fork_client: Arc<ForkClient>,
}

impl<V> ForkedView<V> {
    fn new(inner: V, storage: DataSourceType<OnChain>, fork_client: Arc<ForkClient>) -> Self {
        tracing::debug!("Creating ForkedView");
        Self {
            inner,
            storage,
            fork_client,
        }
    }

    fn persist_value(&self, column: Column, key: Vec<u8>, value: Vec<u8>) {
        let mut column_changes = BTreeMap::new();
        column_changes.insert(
            ReferenceBytesKey::from(key),
            WriteOperation::Insert(value.into()),
        );

        let mut changes = Changes::default();
        changes.insert(column as u32, column_changes);

        if let Err(e) = self
            .storage
            .commit_changes(None, StorageChanges::Changes(changes))
        {
            tracing::warn!(
                "Failed to persist forked value for column {:?}: {}",
                column,
                e
            );
        }
    }
}

impl<V> KeyValueInspect for ForkedView<V>
where
    V: KeyValueInspect<Column = Column>,
{
    type Column = Column;

    fn exists(&self, key: &[u8], column: Self::Column) -> StorageResult<bool> {
        match column {
            Column::ContractsRawCode | Column::ContractsAssets | Column::ContractsState => {
                if self.inner.exists(key, column)? {
                    return Ok(true);
                }

                if column == Column::ContractsRawCode {
                    if let Ok(contract_id) = key.try_into() {
                        let fork_client = self.fork_client.clone();
                        match fork_client.fetch_contract_bytecode_blocking(&contract_id) {
                            Ok(Some(_)) => Ok(true),
                            _ => Ok(false),
                        }
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => self.inner.exists(key, column),
        }
    }

    fn size_of_value(&self, key: &[u8], column: Self::Column) -> StorageResult<Option<usize>> {
        if let Some(size) = self.inner.size_of_value(key, column)? {
            return Ok(Some(size));
        }

        match column {
            Column::ContractsRawCode => {
                if let Ok(contract_id) = key.try_into() {
                    let fork_client = self.fork_client.clone();
                    match fork_client.fetch_contract_bytecode_blocking(&contract_id) {
                        Ok(Some(bytecode)) => Ok(Some(bytecode.len())),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn get(&self, key: &[u8], column: Self::Column) -> StorageResult<Option<Value>> {
        tracing::trace!("ForkedView::get called for column {:?}", column);

        if let Some(value) = self.inner.get(key, column)? {
            tracing::trace!("ForkedView: Found value locally for column {:?}", column);
            return Ok(Some(value));
        }

        tracing::trace!(
            "ForkedView: Value not found locally for column {:?}, checking if contract-related",
            column
        );

        match column {
            Column::ContractsRawCode => {
                if let Ok(contract_id) = key.try_into() {
                    tracing::info!(
                        "ForkedView: Attempting to fetch contract {} from fork",
                        contract_id
                    );
                    let fork_client = self.fork_client.clone();
                    match fork_client.fetch_contract_bytecode_blocking(&contract_id) {
                        Ok(Some(bytecode)) => {
                            tracing::info!(
                                "ForkedView: Successfully fetched contract {} from fork",
                                contract_id
                            );
                            self.persist_value(
                                Column::ContractsRawCode,
                                contract_id.as_ref().to_vec(),
                                bytecode.clone(),
                            );
                            Ok(Some(bytecode.into()))
                        }
                        Ok(None) => {
                            tracing::debug!(
                                "ForkedView: Contract {} not found on fork",
                                contract_id
                            );
                            Ok(None)
                        }
                        Err(e) => {
                            tracing::warn!(
                                "ForkedView: Error fetching contract {} from fork: {}",
                                contract_id,
                                e
                            );
                            Err(fuel_core_storage::Error::Other(e))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Column::ContractsAssets => Ok(None),
            Column::ContractsState => {
                if key.len() >= 64 {
                    let contract_bytes = &key[..32];
                    let state_key_bytes = &key[32..64];

                    if let (Ok(contract_bytes), Ok(state_key_bytes)) = (
                        <[u8; 32]>::try_from(contract_bytes),
                        <[u8; 32]>::try_from(state_key_bytes),
                    ) {
                        let contract_id = ContractId::from(contract_bytes);
                        let state_key = Bytes32::from(state_key_bytes);
                        tracing::info!(
                            "ForkedView: Attempting to fetch state for contract {} key {} from fork",
                            contract_id,
                            state_key
                        );
                        let fork_client = self.fork_client.clone();

                        match fork_client.fetch_contract_state_blocking(&contract_id, &state_key) {
                            Ok(Some(state_data)) => {
                                tracing::info!(
                                    "ForkedView: Successfully fetched state for contract {} key {} from fork",
                                    contract_id,
                                    state_key
                                );
                                let mut storage_key = Vec::with_capacity(
                                    contract_id.as_ref().len() + state_key.as_ref().len(),
                                );
                                storage_key.extend_from_slice(contract_id.as_ref());
                                storage_key.extend_from_slice(state_key.as_ref());
                                self.persist_value(
                                    Column::ContractsState,
                                    storage_key,
                                    state_data.clone(),
                                );
                                Ok(Some(state_data.into()))
                            }
                            Ok(None) => {
                                tracing::debug!(
                                    "ForkedView: State for contract {} key {} not found on fork",
                                    contract_id,
                                    state_key
                                );
                                Ok(None)
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "ForkedView: Error fetching state for contract {} key {} from fork: {}",
                                    contract_id,
                                    state_key,
                                    e
                                );
                                Err(fuel_core_storage::Error::Other(e))
                            }
                        }
                    } else {
                        tracing::warn!("ForkedView: Failed to parse ContractsState key");
                        Ok(None)
                    }
                } else {
                    tracing::warn!(
                        "ForkedView: ContractsState key too short: {} bytes",
                        key.len()
                    );
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn read(
        &self,
        key: &[u8],
        column: Self::Column,
        offset: usize,
        buf: &mut [u8],
    ) -> StorageResult<bool> {
        if self.inner.read(key, column, offset, buf)? {
            return Ok(true);
        }

        match column {
            Column::ContractsRawCode => {
                if let Ok(contract_id) = key.try_into() {
                    let fork_client = self.fork_client.clone();

                    match fork_client.fetch_contract_bytecode_blocking(&contract_id) {
                        Ok(Some(bytecode)) => {
                            if offset >= bytecode.len() {
                                return Ok(false);
                            }
                            let available = bytecode.len() - offset;
                            let len = available.min(buf.len());
                            buf[..len].copy_from_slice(&bytecode[offset..offset + len]);
                            Ok(true)
                        }
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

impl<V> IterableStore for ForkedView<V>
where
    V: IterableStore<Column = Column>,
{
    fn iter_store(
        &self,
        column: Column,
        prefix: Option<&[u8]>,
        start: Option<&[u8]>,
        direction: IterDirection,
    ) -> BoxedIter<KVItem> {
        self.inner.iter_store(column, prefix, start, direction)
    }

    fn iter_store_keys(
        &self,
        column: Column,
        prefix: Option<&[u8]>,
        start: Option<&[u8]>,
        direction: IterDirection,
    ) -> BoxedIter<KeyItem> {
        self.inner.iter_store_keys(column, prefix, start, direction)
    }
}
