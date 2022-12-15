use fuel_types::Bytes64;
use fuels::{
    prelude::*,
    signers::fuel_crypto::{Message, PublicKey, SecretKey, Signature},
    tx::Bytes32,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha3::{Digest, Keccak256};

abigen!(
    EvmEcRecoverContract,
    "test_projects/evm_ec_recover/out/debug/evm_ec_recover-abi.json"
);

fn keccak_hash<B>(data: B) -> Bytes32
where
    B: AsRef<[u8]>,
{
    let mut hasher = Keccak256::new();
    hasher.update(data);
    <[u8; Bytes32::LEN]>::from(hasher.finalize()).into()
}

fn clear_12_bytes(bytes: [u8; 32]) -> [u8; 32] {
    let mut bytes = bytes;
    bytes[..12].copy_from_slice(&[0u8; 12]);

    bytes
}

async fn setup_env() -> Result<(EvmEcRecoverContract, PublicKey, Message, Bytes64, [u8; 32]), Error>
{
    let mut rng = StdRng::seed_from_u64(1000);
    let msg_bytes: Bytes32 = rng.gen();
    let private_key = SecretKey::random(&mut rng);
    let public_key = PublicKey::from(&private_key);

    // generate an "evm address" from the public key
    let pub_key_hash = keccak_hash(*public_key);
    let evm_address = clear_12_bytes(*pub_key_hash);

    let msg = unsafe { Message::from_bytes_unchecked(*msg_bytes) };
    let sig = Signature::sign(&private_key, &msg);
    let sig_bytes: Bytes64 = Bytes64::try_from(sig).unwrap();
    let mut wallet = WalletUnlocked::new_from_private_key(private_key, None);

    let num_assets = 1;
    let coins_per_asset = 10;
    let amount_per_coin = 15;
    let (coins, _asset_ids) = setup_multiple_assets_coins(
        wallet.address(),
        num_assets,
        coins_per_asset,
        amount_per_coin,
    );
    let (provider, _socket_addr) = setup_test_provider(coins.clone(), vec![], None, None).await;
    wallet.set_provider(provider);

    let contract_id = Contract::deploy(
        "test_projects/evm_ec_recover/out/debug/evm_ec_recover.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await?;

    let contract_instance = EvmEcRecoverContract::new(contract_id, wallet.clone());

    Ok((contract_instance, public_key, msg, sig_bytes, evm_address))
}

#[tokio::test]
async fn can_recover_public_key() {
    let (contract, public_key, msg, sig_bytes, _) = setup_env().await.unwrap();
    let sig_r = &sig_bytes[..32];
    let sig_v_s = &sig_bytes[32..];
    let response = contract
        .methods()
        .recover_pub_key(
            Bits256(sig_r.try_into().unwrap()),
            Bits256(sig_v_s.try_into().unwrap()),
            Bits256(msg.into()),
        )
        .call()
        .await
        .unwrap();

    let first = response.value.0;
    let second = response.value.1;
    let arrays: [[u8; 32]; 2] = [first.0, second.0];
    let joined: Vec<u8> = arrays.into_iter().flat_map(|s| s.into_iter()).collect();
    let joined_array: [u8; 64] = joined.try_into().unwrap();
    let pubkey = Bytes64::new(joined_array);

    assert_eq!(pubkey, Bytes64::new(*public_key));
}

#[tokio::test]
async fn can_recover_evm_address() {
    let (contract, _, msg, sig_bytes, evm_address) = setup_env().await.unwrap();
    let sig_r = &sig_bytes[..32];
    let sig_v_s = &sig_bytes[32..];
    let response = contract
        .methods()
        .recover_evm_address(
            Bits256(sig_r.try_into().unwrap()),
            Bits256(sig_v_s.try_into().unwrap()),
            Bits256(*msg),
        )
        .call()
        .await
        .unwrap();

    assert_eq!(response.value, EvmAddress::from(Bits256(evm_address)));
}
