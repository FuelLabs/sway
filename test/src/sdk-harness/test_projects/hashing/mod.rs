use fuels::{
    prelude::*,
    types::{Bits256, SizedAsciiString},
};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

abigen!(Contract(
    name = "HashingTestContract",
    abi = "test_projects/hashing/out/release/hashing-abi.json"
));

enum Hash {
    Sha256,
    Keccak256,
}

fn hash_u8(number: u8, algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(number.to_be_bytes()).into(),
        Hash::Keccak256 => Keccak256::digest(number.to_be_bytes()).into(),
    }
}

fn hash_u16(number: u16, algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(number.to_be_bytes()).into(),
        Hash::Keccak256 => Keccak256::digest(number.to_be_bytes()).into(),
    }
}

fn hash_u32(number: u32, algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(number.to_be_bytes()).into(),
        Hash::Keccak256 => Keccak256::digest(number.to_be_bytes()).into(),
    }
}

fn hash_u64(number: u64, algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(number.to_be_bytes()).into(),
        Hash::Keccak256 => Keccak256::digest(number.to_be_bytes()).into(),
    }
}

fn hash_bool(value: bool, algorithm: Hash) -> [u8; 32] {
    let hash = match algorithm {
        Hash::Sha256 => {
            if value {
                Sha256::digest([1])
            } else {
                Sha256::digest([0])
            }
        }
        Hash::Keccak256 => {
            if value {
                Keccak256::digest([1])
            } else {
                Keccak256::digest([0])
            }
        }
    };

    hash.into()
}

fn hash_str(text: &str, algorithm: Hash) -> [u8; 32] {
    let mut buffer: Vec<u8> = Vec::new();
    for character in text.chars() {
        buffer.push(character as u8);
    }

    match algorithm {
        Hash::Sha256 => Sha256::digest(buffer).into(),
        Hash::Keccak256 => Keccak256::digest(buffer).into(),
    }
}

fn hash_b256(arr: [u8; 32], algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(arr).into(),
        Hash::Keccak256 => Keccak256::digest(arr).into(),
    }
}

fn hash_tuple(arr: [u8; 9], algorithm: Hash) -> [u8; 32] {
    // A tuple is hashed by converting each element into bytes and then combining them together
    // in the sequential order that they are in the tuple
    // E.g. (true, 5) -> [1, 0, 0, 0, 0, 0, 0, 0, 5]
    match algorithm {
        Hash::Sha256 => Sha256::digest(arr).into(),
        Hash::Keccak256 => Keccak256::digest(arr).into(),
    }
}

fn hash_array(arr: [u8; 16], algorithm: Hash) -> [u8; 32] {
    // An array is hashed by converting each element into bytes and then combining them together
    // in the sequential order that they are in the array
    // E.g. (18, 555) -> [0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 2, 43]
    match algorithm {
        Hash::Sha256 => Sha256::digest(arr).into(),
        Hash::Keccak256 => Keccak256::digest(arr).into(),
    }
}

fn hash_enum(arr: [u8; 1], algorithm: Hash) -> [u8; 32] {
    /*
        Enums are encoded in the following format:
        1. Encode the discriminant (the variant tag)
        2. Encode the type of the enum variant

        If all the variants are of type (), or unit,
        then only the discriminant needs to be encoded.

        enum Test {
            A,
            B,
            C
        }

        arr of Test::A will be
        [0]

        arr of Test::B will be
        [1]

        arr of Test::C will be
        [2]
    */
    match algorithm {
        Hash::Sha256 => Sha256::digest(arr).into(),
        Hash::Keccak256 => Keccak256::digest(arr).into(),
    }
}

fn hash_struct(arr: [u8; 55], algorithm: Hash) -> [u8; 32] {
    match algorithm {
        Hash::Sha256 => Sha256::digest(arr).into(),
        Hash::Keccak256 => Keccak256::digest(arr).into(),
    }
}

async fn get_hashing_instance() -> (HashingTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id = Contract::load_from(
        "test_projects/hashing/out/release/hashing.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = HashingTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

mod sha256 {

    use super::*;

    #[tokio::test]
    async fn test_u8() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u8(254, Hash::Sha256));
        let expected_2 = Bits256(hash_u8(253, Hash::Sha256));

        let call_1 = instance.methods().sha256_u8(254u8).call().await.unwrap();
        let call_2 = instance.methods().sha256_u8(254u8).call().await.unwrap();
        let call_3 = instance.methods().sha256_u8(253u8).call().await.unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u16() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u16(65534, Hash::Sha256));
        let expected_2 = Bits256(hash_u16(65533, Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_u16(65534u16)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_u16(65534u16)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_u16(65533u16)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u32() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u32(4294967294, Hash::Sha256));
        let expected_2 = Bits256(hash_u32(4294967293, Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_u32(4294967294u32)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_u32(4294967294u32)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_u32(4294967293u32)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u64() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u64(18446744073709551613, Hash::Sha256));
        let expected_2 = Bits256(hash_u64(18446744073709551612, Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_u64(18446744073709551613)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_u64(18446744073709551613)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_u64(18446744073709551612)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_bool() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_bool(true, Hash::Sha256));
        let expected_2 = Bits256(hash_bool(false, Hash::Sha256));

        let call_1 = instance.methods().sha256_bool(true).call().await.unwrap();
        let call_2 = instance.methods().sha256_bool(true).call().await.unwrap();
        let call_3 = instance.methods().sha256_bool(false).call().await.unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_str() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_str("John", Hash::Sha256));
        let expected_2 = Bits256(hash_str("Nick", Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_str_array(SizedAsciiString::try_from("John").unwrap())
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_str_array(SizedAsciiString::try_from("John").unwrap())
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_str_array(SizedAsciiString::try_from("Nick").unwrap())
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_b256() {
        let (instance, _id) = get_hashing_instance().await;

        let address1 = [
            118, 64, 238, 245, 229, 5, 191, 187, 201, 174, 141, 75, 72, 119, 88, 252, 38, 62, 110,
            176, 51, 16, 126, 190, 233, 136, 54, 127, 90, 101, 230, 168,
        ];
        let address2 = [
            8, 4, 28, 217, 200, 5, 161, 17, 20, 214, 54, 77, 72, 118, 90, 31, 225, 63, 110, 77,
            190, 190, 12, 1, 233, 48, 54, 72, 90, 253, 100, 103,
        ];

        let expected_1 = Bits256(hash_b256(address1, Hash::Sha256));
        let expected_2 = Bits256(hash_b256(address2, Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_b256(Bits256(address1))
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_b256(Bits256(address1))
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_b256(Bits256(address2))
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_tuple() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_tuple([1, 0, 0, 0, 0, 0, 0, 0, 5], Hash::Sha256));
        let expected_2 = Bits256(hash_tuple([1, 0, 0, 0, 0, 0, 0, 0, 6], Hash::Sha256));

        let call_1 = instance
            .methods()
            .sha256_tuple((true, 5))
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .sha256_tuple((true, 5))
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .sha256_tuple((true, 6))
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_array() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_array(
            [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5],
            Hash::Sha256,
        ));
        let expected_2 = Bits256(hash_array(
            [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 6],
            Hash::Sha256,
        ));

        let call_1 = instance.methods().sha256_array(1, 5).call().await.unwrap();
        let call_2 = instance.methods().sha256_array(1, 5).call().await.unwrap();
        let call_3 = instance.methods().sha256_array(1, 6).call().await.unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_enum() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_enum([0], Hash::Sha256));
        let expected_2 = Bits256(hash_enum([1], Hash::Sha256));

        let call_1 = instance.methods().sha256_enum(true).call().await.unwrap();
        let call_2 = instance.methods().sha256_enum(true).call().await.unwrap();
        let call_3 = instance.methods().sha256_enum(false).call().await.unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_struct() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_struct(
            [
                74, 111, 104, 110, 18, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 9, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ],
            Hash::Sha256,
        ));
        let expected_2 = Bits256(hash_struct(
            [
                74, 111, 104, 110, 18, 1, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 9, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ],
            Hash::Sha256,
        ));

        let call_1 = instance.methods().sha256_struct(true).call().await.unwrap();
        let call_2 = instance.methods().sha256_struct(true).call().await.unwrap();
        let call_3 = instance
            .methods()
            .sha256_struct(false)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }
}

mod keccak256 {

    use super::*;

    #[tokio::test]
    async fn test_u8() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u8(254, Hash::Keccak256));
        let expected_2 = Bits256(hash_u8(253, Hash::Keccak256));

        let call_1 = instance.methods().keccak256_u8(254u8).call().await.unwrap();
        let call_2 = instance.methods().keccak256_u8(254u8).call().await.unwrap();
        let call_3 = instance.methods().keccak256_u8(253u8).call().await.unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u16() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u16(65534, Hash::Keccak256));
        let expected_2 = Bits256(hash_u16(65533, Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_u16(65534u16)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_u16(65534u16)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_u16(65533u16)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u32() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u32(4294967294, Hash::Keccak256));
        let expected_2 = Bits256(hash_u32(4294967293, Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_u32(4294967294u32)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_u32(4294967294u32)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_u32(4294967293u32)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_u64() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_u64(18446744073709551613, Hash::Keccak256));
        let expected_2 = Bits256(hash_u64(18446744073709551612, Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_u64(18446744073709551613)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_u64(18446744073709551613)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_u64(18446744073709551612)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_bool() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_bool(true, Hash::Keccak256));
        let expected_2 = Bits256(hash_bool(false, Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_bool(true)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_bool(true)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_bool(false)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_str() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_str("John", Hash::Keccak256));
        let expected_2 = Bits256(hash_str("Nick", Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_str(SizedAsciiString::try_from("John").unwrap())
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_str(SizedAsciiString::try_from("John").unwrap())
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_str(SizedAsciiString::try_from("Nick").unwrap())
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_b256() {
        let (instance, _id) = get_hashing_instance().await;

        let address1 = [
            118, 64, 238, 245, 229, 5, 191, 187, 201, 174, 141, 75, 72, 119, 88, 252, 38, 62, 110,
            176, 51, 16, 126, 190, 233, 136, 54, 127, 90, 101, 230, 168,
        ];
        let address2 = [
            8, 4, 28, 217, 200, 5, 161, 17, 20, 214, 54, 77, 72, 118, 90, 31, 225, 63, 110, 77,
            190, 190, 12, 1, 233, 48, 54, 72, 90, 253, 100, 103,
        ];

        let expected_1 = Bits256(hash_b256(address1, Hash::Keccak256));
        let expected_2 = Bits256(hash_b256(address2, Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_b256(Bits256(address1))
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_b256(Bits256(address1))
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_b256(Bits256(address2))
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_tuple() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_tuple([1, 0, 0, 0, 0, 0, 0, 0, 5], Hash::Keccak256));
        let expected_2 = Bits256(hash_tuple([1, 0, 0, 0, 0, 0, 0, 0, 6], Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_tuple((true, 5))
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_tuple((true, 5))
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_tuple((true, 6))
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_array() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_array(
            [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5],
            Hash::Keccak256,
        ));
        let expected_2 = Bits256(hash_array(
            [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 6],
            Hash::Keccak256,
        ));

        let call_1 = instance
            .methods()
            .keccak256_array(1, 5)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_array(1, 5)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_array(1, 6)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_enum() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_enum([0], Hash::Keccak256));
        let expected_2 = Bits256(hash_enum([1], Hash::Keccak256));

        let call_1 = instance
            .methods()
            .keccak256_enum(true)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_enum(true)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_enum(false)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }

    #[tokio::test]
    async fn test_struct() {
        let (instance, _id) = get_hashing_instance().await;

        let expected_1 = Bits256(hash_struct(
            [
                74, 111, 104, 110, 18, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 9, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ],
            Hash::Keccak256,
        ));
        let expected_2 = Bits256(hash_struct(
            [
                74, 111, 104, 110, 18, 1, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 9, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ],
            Hash::Keccak256,
        ));

        let call_1 = instance
            .methods()
            .keccak256_struct(true)
            .call()
            .await
            .unwrap();
        let call_2 = instance
            .methods()
            .keccak256_struct(true)
            .call()
            .await
            .unwrap();
        let call_3 = instance
            .methods()
            .keccak256_struct(false)
            .call()
            .await
            .unwrap();

        assert_eq!(call_1.value, call_2.value);
        assert_ne!(call_1.value, call_3.value);

        assert_eq!(expected_1, call_1.value);
        assert_eq!(expected_2, call_3.value);
    }
}
