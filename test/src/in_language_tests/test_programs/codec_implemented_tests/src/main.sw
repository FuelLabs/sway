library;
// Logs every new type defined in the std lib to ensure codec is working for them

use std::{
    address::Address,
    alias::SubId,
    asset_id::AssetId,
    auth::AuthError,
    b512::B512,
    block::BlockHashError,
    bytes::Bytes,
    contract_id::ContractId,
    crypto::{
        alt_bn128::AltBn128Error,
        ed25519::Ed25519,
        message::Message,
        point2d::Point2D,
        public_key::PublicKey,
        scalar::Scalar,
        secp256k1::Secp256k1,
        secp256r1::Secp256r1,
        signature::Signature,
        signature_error::SignatureError,
    },
    ecr::EcRecoverError,
    hash::Hasher,
    identity::Identity,
    inputs::Input,
    logging::log,
    low_level_call::CallParams,
    option::Option,
    outputs::Output,
    result::Result,
    storage::{
        storage_bytes::StorageBytes,
        storage_key::StorageKey,
        storage_string::StorageString,
        storage_vec::StorageVec,
    },
    string::String,
    tx::Transaction,
    u128::U128,
    vec::Vec,
    vm::evm::evm_address::EvmAddress,
};
#[test]

fn test_logging() {
    log(Address::zero());
    log(Bytes::new());
    log(AltBn128Error::InvalidEllipticCurvePoint);
    log(Ed25519::new());
    log(Message::new());
    log(Point2D::new());
    log(PublicKey::new());
    log(Scalar::new());
    log(Secp256k1::new());
    log(Secp256r1::new());
    log(SignatureError::UnrecoverablePublicKey);
    log(Signature::Secp256k1(Secp256k1::new()));
    log(StorageBytes {});
    let skey: StorageKey<u64> = StorageKey::new(b256::zero(), 0, b256::zero());
    log(skey);
    let smap: StorageMap<u64, u64> = StorageMap {};
    log(smap);
    log(StorageString {});
    let svec: StorageVec<u64> = StorageVec {};
    log(svec);
    log(EvmAddress::zero());
    log(SubId::zero());
    log(Vec::<u64>::new());
    log(AssetId::zero());
    log(AuthError::CallerIsInternal);
    log(B512::zero());
    log(BlockHashError::BlockHeightTooHigh);
    log(ContractId::zero());
    log(EcRecoverError::UnrecoverablePublicKey);
    log(Hasher::new());
    log(Identity::Address(Address::zero()));
    log(Input::Coin);
    log(CallParams {
        coins: 0,
        asset_id: AssetId::zero(),
        gas: 0,
    });
    let option: Option<u64> = Option::Some(0);
    log(option);
    log(Output::Coin);
    let res: Result<u64, u64> = Result::Ok(0);
    log(res);
    log(String::new());
    log(Transaction::Script);
    log(U128::zero());
}
