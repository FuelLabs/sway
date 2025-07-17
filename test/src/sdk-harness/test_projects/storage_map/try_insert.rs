use super::*;

#[macro_export]
macro_rules! generate_try_insert_tests {
    ($input_string:expr, $key:expr, $value1:expr, $value2:expr) => {
        paste::paste! {
            #[tokio::test]
            async fn [<try_insert_ $input_string _exists>]() {
                let instance = test_storage_map_instance().await;

                instance.methods().[<insert_into_ $input_string _map>]($key, $value1).call().await.unwrap();

                let prev = instance.methods().[<get_from_ $input_string _map>]($key).call().await.unwrap().value;

                assert_eq!(prev, Some($value1));

                let result = instance.methods().[<try_insert_into_ $input_string _map>]($key, $value2).call().await.unwrap().value;

                assert_eq!(result, Err(StorageMapError::OccupiedError($value1)));

                let after = instance.methods().[<get_from_ $input_string _map>]($key).call().await.unwrap().value;

                assert_eq!(after, Some($value1));
            }

            #[tokio::test]
            async fn [<try_insert_ $input_string _does_not_exist>]() {
                let instance = test_storage_map_instance().await;

                let prev = instance.methods().[<get_from_ $input_string _map>]($key).call().await.unwrap().value;

                assert_eq!(prev, None);

                let result = instance.methods().[<try_insert_into_ $input_string _map>]($key, $value2).call().await.unwrap().value;

                assert_eq!(result, Ok($value2));

                let after = instance.methods().[<get_from_ $input_string _map>]($key).call().await.unwrap().value;

                assert_eq!(after, Some($value2));
            }
        }
    };
}

generate_try_insert_tests!(u64_to_bool, 1, true, false);
generate_try_insert_tests!(u64_to_u8, 1, 1, 2);
generate_try_insert_tests!(u64_to_u16, 1, 1, 2);
generate_try_insert_tests!(u64_to_u32, 1, 1, 2);
generate_try_insert_tests!(u64_to_u64, 1, 1, 2);
generate_try_insert_tests!(
    u64_to_tuple,
    1,
    (Bits256([1; 32]), 1, true),
    (Bits256([2; 32]), 2, false)
);
generate_try_insert_tests!(
    u64_to_struct,
    1,
    Struct {
        x: 1,
        y: Bits256([1; 32]),
        z: Bits256([2; 32])
    },
    Struct {
        x: 2,
        y: Bits256([3; 32]),
        z: Bits256([4; 32])
    }
);
generate_try_insert_tests!(u64_to_enum, 1, Enum::V1(Bits256([1; 32])), Enum::V2(2));
generate_try_insert_tests!(
    u64_to_str,
    1,
    SizedAsciiString::try_from("aaaaaaaaaA").unwrap(),
    SizedAsciiString::try_from("bbbbbbbbbB").unwrap()
);
generate_try_insert_tests!(
    u64_to_array,
    1,
    [Bits256([1; 32]); 3],
    [Bits256([2; 32]); 3]
);
generate_try_insert_tests!(bool_to_u64, true, 1, 2);
generate_try_insert_tests!(u8_to_u64, 1, 1, 2);
generate_try_insert_tests!(u16_to_u64, 1, 1, 2);
generate_try_insert_tests!(u32_to_u64, 1, 1, 2);
generate_try_insert_tests!(tuple_to_u64, (Bits256([1; 32]), 1, true), 1, 2);
generate_try_insert_tests!(
    struct_to_u64,
    Struct {
        x: 1,
        y: Bits256([1; 32]),
        z: Bits256([2; 32])
    },
    1,
    2
);
generate_try_insert_tests!(enum_to_u64, Enum::V1(Bits256([1; 32])), 1, 2);
generate_try_insert_tests!(
    str_to_u64,
    SizedAsciiString::try_from("aaaaaaaaaA").unwrap(),
    1,
    2
);
generate_try_insert_tests!(array_to_u64, [Bits256([1; 32]); 3], 1, 2);
