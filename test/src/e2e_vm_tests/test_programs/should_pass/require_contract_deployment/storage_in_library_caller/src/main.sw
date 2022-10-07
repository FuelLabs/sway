script;

use storage_in_library_abi::StorageInLibrary;

fn main() -> bool {
    let storage_in_library_contract_id = 0x649b482bd4ee2f1515a9cb42a561fa732e7bb767d32aabf351548c5c1cd1f1db;

    let instance = abi(StorageInLibrary, storage_in_library_contract_id);

    instance.call_update_library_storage();
    
    assert(instance.call_get_library_storage() == 70);

    true
}
