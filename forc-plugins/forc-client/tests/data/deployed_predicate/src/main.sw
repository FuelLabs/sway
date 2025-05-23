predicate;

#[allow(dead_code)]
enum EnumWithGeneric<D> {
    VariantOne: D,
    VariantTwo: (),
}

struct StructWithGeneric<D> {
    field_1: D,
    field_2: u64,
}

impl<D> PartialEq for EnumWithGeneric<D> 
where
    D: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumWithGeneric::VariantOne(d1), EnumWithGeneric::VariantOne(d2)) => d1 == d2,
            (EnumWithGeneric::VariantTwo, EnumWithGeneric::VariantTwo) => true,
            _ => false,
        }
    }
}

impl<D> PartialEq for StructWithGeneric<D>
where
    D: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        self.field_1 == other.field_1 && self.field_2 == other.field_2
    }
}

configurable {
    BOOL: bool = true,
    U8: u8 = 8,
    STRUCT: StructWithGeneric<u8> = StructWithGeneric {
        field_1: 8,
        field_2: 16,
    },
    ENUM: EnumWithGeneric<bool> = EnumWithGeneric::VariantOne(true),
}

fn main(
    switch: bool,
    u_8: u8,
    some_struct: StructWithGeneric<u8>,
    some_enum: EnumWithGeneric<bool>,
) -> bool {
    switch == BOOL && u_8 == U8 && some_struct == STRUCT && some_enum == ENUM
}
