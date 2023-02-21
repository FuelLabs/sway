script;

struct MyType {
    x: AssetId,
}

type MyIdentity = Identity;

type MyTypeAlias1 = MyType;

type MyTypeAlias2 = MyTypeAlias1;

fn foo(x: AssetId) -> AssetId { 
    AssetId::from(x.value)
}

fn main() -> bool {
    let x = AssetId { value: 0x0000000000000000000000000000000000000000000000000000000000000001 };
    let y: AssetId = x;
    let z = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    foo(x); 
    let t = MyTypeAlias1 { 
        x: AssetId { 
            value:  0x0000000000000000000000000000000000000000000000000000000000000001
        } 
    };
    
    let o = Option::Some(x);
    if let Option::Some(AssetId { value }) = o {
        assert(value == z.value);
    }

    let value = match o {
        Option::Some(value) => value.value,
        Option::None => revert(42),
    };

    let id = MyIdentity::ContractId(x);
    /* match id {
        MyIdentity::ContractId(AssetId { value }) => assert(value == 0x0000000000000000000000000000000000000000000000000000000000000001),
        _ => revert(42),
    }*/

    let id = MyIdentity::Address(Address::from(0x1111111111111111111111111111111111111111111111111111111111111111));
    /* match id {
        MyIdentity::Address(Address { value }) => assert(value == 0x1111111111111111111111111111111111111111111111111111111111111111),
        _ => revert(42),
    }*/

    assert(value == 0x0000000000000000000000000000000000000000000000000000000000000001);

    x == z && t.x.value == y.value && z.value == y.value
}
