use fuels::{prelude::*, types::Bits256};

abigen!(Contract(
    name = "TestMessagesContract",
    abi = "test_projects/messages/out/release/messages-abi.json"
));

async fn get_messages_contract_instance() -> (TestMessagesContract<Wallet>, ContractId, Wallet) {
    let num_wallets = 1;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );

    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let messages_contract_id = Contract::load_from(
        "test_projects/messages/out/release/messages.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallets[0], TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    // Send assets to the contract to be able withdraw via `smo`.
    wallets[0]
        .force_transfer_to_contract(
            messages_contract_id,
            amount_per_coin >> 1,
            AssetId::BASE,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    let messages_instance =
        TestMessagesContract::new(messages_contract_id.clone(), wallets[0].clone());

    (
        messages_instance,
        messages_contract_id.into(),
        wallets[0].clone(),
    )
}

#[tokio::test]
async fn can_send_bool_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = true;
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_bool(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(9, message_receipt.len().unwrap()); // smo ID + 1 bytes
    assert_eq!(vec![1], message_receipt.data().unwrap()[8..9]);
}

#[tokio::test]
async fn can_send_u8_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = 42u8;
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_u8(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(9, message_receipt.len().unwrap()); // smo ID + 8 bytes
    assert_eq!(vec![42], message_receipt.data().unwrap()[8..9]);
}

#[tokio::test]
async fn can_send_u16_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = 42u16;
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_u16(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(16, message_receipt.len().unwrap()); // smo ID + 8 bytes
    assert_eq!(
        vec![0, 0, 0, 0, 0, 0, 0, 42],
        message_receipt.data().unwrap()[8..16]
    );
}

#[tokio::test]
async fn can_send_u32_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = 42u32;
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_u32(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(16, message_receipt.len().unwrap()); // smo ID + 8 bytes
    assert_eq!(
        vec![0, 0, 0, 0, 0, 0, 0, 42],
        message_receipt.data().unwrap()[8..16]
    );
}

#[tokio::test]
async fn can_send_u64_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = 42u64;
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_u64(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(16, message_receipt.len().unwrap()); // smo ID + 8 bytes
    assert_eq!(
        vec![0, 0, 0, 0, 0, 0, 0, 42],
        message_receipt.data().unwrap()[8..16]
    );
}

#[tokio::test]
async fn can_send_b256_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = [1u8; 32];
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_b256(Bits256(*recipient_address), Bits256(message), amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(40, message_receipt.len().unwrap()); // smo ID + 32 bytes
    assert_eq!(message.to_vec(), message_receipt.data().unwrap()[8..40]);
}

#[tokio::test]
async fn can_send_struct_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = MyStruct {
        first_field: 42,
        second_field: 69,
    };
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_struct(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(24, message_receipt.len().unwrap()); // smo ID + 16 bytes
    assert_eq!(
        [
            0, 0, 0, 0, 0, 0, 0, 42, // first field
            0, 0, 0, 0, 0, 0, 0, 69, // second field
        ],
        message_receipt.data().unwrap()[8..24]
    );
}

#[tokio::test]
async fn can_send_enum_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = MyEnum::<Bits256>::SecondVariant(42);
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_enum(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(48, message_receipt.len().unwrap()); // smo ID + 8 bytes (tag) + 32 bytes (largest variant)
    assert_eq!(
        [
            0, 0, 0, 0, 0, 0, 0, 1, // tag
            0, 0, 0, 0, 0, 0, 0, 0, // padding
            0, 0, 0, 0, 0, 0, 0, 0, // padding
            0, 0, 0, 0, 0, 0, 0, 0, // padding
            0, 0, 0, 0, 0, 0, 0, 42, // padding
        ],
        message_receipt.data().unwrap()[8..48]
    );
}

#[tokio::test]
async fn can_send_array_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = [42, 43, 44];
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_array(Bits256(*recipient_address), message, amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(32, message_receipt.len().unwrap()); // smo ID + 24 bytes
    assert_eq!(
        [
            0, 0, 0, 0, 0, 0, 0, 42, // first element
            0, 0, 0, 0, 0, 0, 0, 43, // second element
            0, 0, 0, 0, 0, 0, 0, 44, // third element
        ],
        message_receipt.data().unwrap()[8..32]
    );
}

#[tokio::test]
async fn can_send_string_message() {
    let (messages_instance, messages_contract_id, wallet) = get_messages_contract_instance().await;
    let recipient_address: Address = wallet.address().into();
    let message = "fuel";
    let amount = 33u64;

    let call_response = messages_instance
        .methods()
        .send_typed_message_string(
            Bits256(*recipient_address),
            message.try_into().unwrap(),
            amount,
        )
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*messages_contract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(16, message_receipt.len().unwrap()); // smo ID + 4 bytes
    assert_eq!(
        [
            102, // 'f'
            117, // 'u'
            101, // 'e'
            108, // 'l'
        ],
        message_receipt.data().unwrap()[8..12]
    );
}
