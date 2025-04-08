use fuels::{prelude::*, types::Bits256};

abigen!(Contract(
    name = "GenericsInAbiTestContract",
    abi = "test_projects/generics_in_abi/out/release/generics_in_abi-abi.json"
));

async fn get_generics_in_abi_instance() -> (GenericsInAbiTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/generics_in_abi/out/release/generics_in_abi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = GenericsInAbiTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn generics_bool() -> Result<()> {
    let (instance, _id) = get_generics_in_abi_instance().await;
    let contract_methods = instance.methods();

    {
        // simple struct with a single generic param
        let arg1 = SimpleGeneric {
            single_generic_param: 123u64,
        };

        let result = contract_methods
            .struct_w_generic(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // struct that delegates the generic param internally
        let arg1 = PassTheGenericOn {
            one: SimpleGeneric {
                single_generic_param: "abc".try_into()?,
            },
        };

        let result = contract_methods
            .struct_delegating_generic(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // struct that has the generic in an array
        let arg1 = StructWArrayGeneric { a: [1u32, 2u32] };

        let result = contract_methods
            .struct_w_generic_in_array(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // struct that has the generic in a tuple
        let arg1 = StructWTupleGeneric { a: (1, 2) };

        let result = contract_methods
            .struct_w_generic_in_tuple(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // struct that has the generic in a tuple
        let arg1 = StructWDiffTupleGeneric { a: (1, false) };

        let result = contract_methods
            .struct_w_diff_generic_in_tuple(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // struct with generic in variant
        let arg1 = EnumWGeneric::b(10);
        let result = contract_methods
            .enum_w_generic(arg1.clone())
            .call()
            .await?
            .value;

        assert_eq!(result, arg1);
    }
    {
        // complex case
        let pass_through = PassTheGenericOn {
            one: SimpleGeneric {
                single_generic_param: "ab".try_into()?,
            },
        };
        let w_arr_generic = StructWArrayGeneric {
            a: [pass_through.clone(), pass_through],
        };

        let arg1 = MegaExample {
            a: ([Bits256([0; 32]), Bits256([0; 32])], "ab".try_into()?),
            b: vec![(
                [EnumWGeneric::b(StructWTupleGeneric {
                    a: (w_arr_generic.clone(), w_arr_generic),
                })],
                10u32,
            )],
        };

        contract_methods.complex_test(arg1.clone()).call().await?;
    }

    Ok(())
}
