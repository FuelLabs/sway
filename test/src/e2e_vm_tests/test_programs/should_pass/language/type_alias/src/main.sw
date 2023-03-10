script;

struct MyType {
    x: std::contract_id::AssetId,
}
type MyTypeAlias1 = MyType;
type MyTypeAlias2 = MyTypeAlias1;
type MyTypeAlias3 = MyTypeAlias1;
type TupleAlias = (MyTypeAlias1, MyTypeAlias2);
type MyU64 = u64;

impl MyType {
    fn bar0(self) -> u64 { 0 }
}

impl MyTypeAlias1 {
    fn bar1(self) -> u64 { 1 }
}

impl MyTypeAlias2 {
    fn bar2(self) -> u64 { 2 }
}

impl MyTypeAlias3 {
    fn bar3(self) -> u64 { 3 }
}


impl core::ops::Eq for MyTypeAlias2 {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}

struct GenericStruct<T> {
    x: T
}

fn foo(x: AssetId) -> AssetId { 
    AssetId::from(x.value)
}

fn struct_tests() {
    /* Structs */
    let x = AssetId { value: 0x0000000000000000000000000000000000000000000000000000000000000001 };
    let y: AssetId = x;
    let z = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    foo(x); 
    let t = MyTypeAlias2 { 
        x: std::contract_id::AssetId { 
            value:  0x0000000000000000000000000000000000000000000000000000000000000001
        } 
    };
    let t2 = MyTypeAlias3 { 
        x: AssetId { 
            value:  0x0000000000000000000000000000000000000000000000000000000000000001
        } 
    };
    assert(x == z && t.x.value == y.value && t.x.value == t2.x.value && z.value == y.value);

    assert(t.bar0() == 0 && t.bar1() == 1 && t.bar2() == 2 && t.bar3() == 3);
    assert(t2.bar0() == 0 && t2.bar1() == 1 && t2.bar2() == 2 && t2.bar3() == 3);
    assert(t == t2);

    let s: GenericStruct<MyU64> = GenericStruct { 
        x: 42
    };
    assert(s.x == 42);

    let s: GenericStruct<MyTypeAlias3> = GenericStruct { 
        x: t
    };
    assert(s.x == t);

    let tuple: TupleAlias = (t, t);
    assert(tuple.0 == t);
    assert(tuple.1 == t);
}

type MyIdentity = Identity;
fn noop1(x: MyIdentity) -> MyIdentity { 
    match x {
        Identity::ContractId(a) => MyIdentity::ContractId(a),
        Identity::Address(a) => MyIdentity::Address(a),
    }
}

fn noop2(x: Identity) -> Identity { 
    match x {
        Identity::ContractId(a) => Identity::ContractId(a),
        Identity::Address(a) => Identity::Address(a),
    }
}

enum MyEnumType {
    X: std::contract_id::AssetId,
}
type MyEnumTypeAlias1 = MyEnumType;
type MyEnumTypeAlias2 = MyEnumTypeAlias1;
type MyEnumTypeAlias3 = MyEnumTypeAlias1;

impl MyEnumType {
    fn bar0(self) -> u64 { 0 }
}

impl MyEnumTypeAlias1 {
    fn bar1(self) -> u64 { 1 }
}

impl MyEnumTypeAlias2 {
    fn bar2(self) -> u64 { 2 }
}

impl MyEnumTypeAlias3 {
    fn bar3(self) -> u64 { 3 }
}

impl core::ops::Eq for MyEnumTypeAlias2 {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (MyEnumType::X(value1), MyEnumType::X(value2)) => value1 == value2,
        }
    }
}

fn enum_tests() {
    let x = ContractId { value: 0x0000000000000000000000000000000000000000000000000000000000000001 };
    let z = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let o = Option::Some(x);
    if let Option::Some(AssetId { value }) = o {
        assert(value == z.value);
    }

    let value = match o {
        Option::Some(value) => value.value,
        Option::None => revert(42),
    };

    let id1 = MyIdentity::ContractId(x);
    let id2 = MyIdentity::ContractId(x);
    match id1 {
        Identity::ContractId(AssetId { value }) => assert(value == 0x0000000000000000000000000000000000000000000000000000000000000001), // Support MyIdentity here
        _ => revert(42),
    }
    assert(id1 == id2); // test trait `Eq`

    let id3 = MyIdentity::Address(Address::from(0x1111111111111111111111111111111111111111111111111111111111111111));
    let id4 = MyIdentity::Address(Address::from(0x1111111111111111111111111111111111111111111111111111111111111111));
    match id3 {
        Identity::Address(Address { value }) => assert(value == 0x1111111111111111111111111111111111111111111111111111111111111111), // Support MyIdentity here
        _ => revert(42),
    }
    assert(id3 == id4);

    assert(id1 == noop1(id1));
    assert(noop1(id1) == id1);

    assert(id3 == noop2(id3));
    assert(noop2(id3) == id3);

    assert(value == 0x0000000000000000000000000000000000000000000000000000000000000001);

    let e1 = MyEnumTypeAlias3::X(z);
    let e2 = MyEnumTypeAlias2::X(z);
    assert(e1.bar0() == 0 && e1.bar1() == 1 && e1.bar2() == 2 && e1.bar3() == 3);
    assert(e2.bar0() == 0 && e2.bar1() == 1 && e2.bar2() == 2 && e2.bar3() == 3);
    assert(e1 == e2);
}

fn main() {
    struct_tests();
    enum_tests();
}
