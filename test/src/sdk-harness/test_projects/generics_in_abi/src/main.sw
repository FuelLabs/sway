contract;

use std::hash::*;

struct SimpleGeneric<T> {
    single_generic_param: T,
}

struct PassTheGenericOn<K> {
    one: SimpleGeneric<K>,
}

struct StructWArrayGeneric<L> {
    a: [L; 2],
}

struct StructWTupleGeneric<M> {
    a: (M, M),
}

struct StructWDiffTupleGeneric<M, N> {
    a: (M, N),
}

enum EnumWGeneric<N> {
    a: u64,
    b: N,
}

struct MegaExample<T, U> {
    a: ([U; 2], T),
    b: Vec<([EnumWGeneric<StructWTupleGeneric<StructWArrayGeneric<PassTheGenericOn<T>>>>; 1], u32)>,
}

abi MyContract {
    fn struct_w_generic(arg1: SimpleGeneric<u64>) -> SimpleGeneric<u64>;
    fn struct_delegating_generic(arg1: PassTheGenericOn<str[3]>) -> PassTheGenericOn<str[3]>;
    fn struct_w_generic_in_array(arg1: StructWArrayGeneric<u32>) -> StructWArrayGeneric<u32>;
    fn struct_w_generic_in_tuple(arg1: StructWTupleGeneric<u32>) -> StructWTupleGeneric<u32>;
    fn struct_w_diff_generic_in_tuple(
        arg1: StructWDiffTupleGeneric<u32, bool>,
    ) -> StructWDiffTupleGeneric<u32, bool>;

    fn enum_w_generic(arg1: EnumWGeneric<u64>) -> EnumWGeneric<u64>;

    fn complex_test(arg1: MegaExample<str[2], b256>);
}

impl MyContract for Contract {
    fn struct_w_generic(arg1: SimpleGeneric<u64>) -> SimpleGeneric<u64> {
        let expected = SimpleGeneric {
            single_generic_param: 123u64,
        };

        assert(arg1.single_generic_param == expected.single_generic_param);

        expected
    }

    fn struct_delegating_generic(arg1: PassTheGenericOn<str[3]>) -> PassTheGenericOn<str[3]> {
        let expected = PassTheGenericOn {
            one: SimpleGeneric {
                single_generic_param: __to_str_array("abc"),
            },
        };

        assert(
            sha256(expected.one.single_generic_param) == sha256(arg1.one.single_generic_param),
        );

        expected
    }

    fn struct_w_generic_in_array(arg1: StructWArrayGeneric<u32>) -> StructWArrayGeneric<u32> {
        let expected = StructWArrayGeneric {
            a: [1u32, 2u32],
        };

        assert(expected.a[0] == arg1.a[0]);
        assert(expected.a[1] == arg1.a[1]);

        expected
    }

    fn struct_w_generic_in_tuple(arg1: StructWTupleGeneric<u32>) -> StructWTupleGeneric<u32> {
        let expected = StructWTupleGeneric {
            a: (1u32, 2u32),
        };
        assert(expected.a.0 == arg1.a.0);
        assert(expected.a.1 == arg1.a.1);

        expected
    }

    fn struct_w_diff_generic_in_tuple(
        arg1: StructWDiffTupleGeneric<u32, bool>,
    ) -> StructWDiffTupleGeneric<u32, bool> {
        let expected = StructWDiffTupleGeneric {
            a: (1u32, false),
        };
        assert(expected.a.0 == arg1.a.0);
        assert(expected.a.1 == arg1.a.1);

        expected
    }
    fn enum_w_generic(arg1: EnumWGeneric<u64>) -> EnumWGeneric<u64> {
        match arg1 {
            EnumWGeneric::b(value) => {
                assert(value == 10u64);
            }
            _ => {
                assert(false)
            }
        }
        EnumWGeneric::b(10)
    }

    fn complex_test(_arg1: MegaExample<str[2], b256>) {}
}
