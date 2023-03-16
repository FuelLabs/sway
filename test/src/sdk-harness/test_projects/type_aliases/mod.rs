use fuels::{
    prelude::*,
    types::{Identity, SizedAsciiString},
};

abigen!(Contract(
    name = "TypeAliasesTestContract",
    abi = "test_projects/type_aliases/out/debug/type_aliases-abi.json"
));

async fn get_type_aliases_instance() -> (TypeAliasesTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/type_aliases/out/debug/type_aliases.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/type_aliases/out/debug/type_aliases-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = TypeAliasesTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_foo() -> Result<()> {
    let (instance, _id) = get_type_aliases_instance().await;
    let contract_methods = instance.methods();

    let x = ContractId::new([1u8; 32]);

    let y = [Identity::ContractId(x), Identity::ContractId(x)];

    let z = IdentityAliasWrapper { i: y[0].clone() };

    let w = Generic { f: z.clone() };

    let u = (x, x);

    let s = SizedAsciiString::try_from("fuelfuel0").unwrap();

    let (x_result, y_result, z_result, w_result, u_result, s_result) = contract_methods
        .foo(x, y.clone(), z.clone(), w.clone(), u.clone(), s.clone())
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(x, x_result);
    assert_eq!(y, y_result);
    assert_eq!(z, z_result);
    assert_eq!(w, w_result);
    assert_eq!(u, u_result);
    assert_eq!(s, s_result);

    Ok(())
}
