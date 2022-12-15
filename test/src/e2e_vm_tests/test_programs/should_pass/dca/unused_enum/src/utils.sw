library utils;

dep r#enum;
use r#enum::AnEnum;

pub fn uses_enum() -> AnEnum {
    AnEnum::Variant
}
