use fuels::{
    prelude::*,
    types::{Bits256, Identity, SizedAsciiString},
};

abigen!(Contract(
    name = "TypeAliasesTestContract",
    abi = "out/type_aliases-abi.json"
));

async fn get_type_aliases_instance() -> (TypeAliasesTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/type_aliases.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = TypeAliasesTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_foo() -> Result<()> {
    let (instance, _id) = get_type_aliases_instance().await;
    let contract_methods = instance.methods();

    let x = Bits256([1u8; 32]);

    let y = [
        Identity::ContractId(ContractId::new([1u8; 32])),
        Identity::ContractId(ContractId::new([1u8; 32])),
    ];

    let z = IdentityAliasWrapper { i: y[0].clone() };

    let w = Generic { f: z.clone() };

    let u = (x, x);

    let s = SizedAsciiString::try_from("fuelfuel0").unwrap();

    let (x_result, y_result, z_result, w_result, u_result, s_result) = contract_methods
        .foo(x, y.clone(), z.clone(), w.clone(), u, s.clone())
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
