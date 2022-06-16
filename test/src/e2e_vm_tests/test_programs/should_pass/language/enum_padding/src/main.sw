script;

pub enum LowerLevelEnum {
    first: b256,
    second: u32,
}

pub struct ThenAStruct {
    first: u32,
    second: LowerLevelEnum,
}

pub enum TopLevelEnum {
    first: (b256,
    b256), second: ThenAStruct,
}

fn main() -> TopLevelEnum {
    // Expected output:
    //
    //  0000000000000001  # TopLevelEnum.tag
    //  0000000000000000  #     TopLevelEnum.padding
    //  0000000000000000  #     TopLevelEnum.padding
    //  000000000000002a  #     ThenAStruct.first(42)
    //  0000000000000001  #     ThenAStruct.LowerLevelEnum.tag
    //  0000000000000000  #         ThenAStruct.LowerLevelEnum.padding
    //  0000000000000000  #         ThenAStruct.LowerLevelEnum.padding
    //  0000000000000000  #         ThenAStruct.LowerLevelEnum.padding
    //  0000000000000042  #         ThenAStruct.LowerLevelEnum.second(66)

    TopLevelEnum::second(ThenAStruct {
        first: 42, second: LowerLevelEnum::second(66)
    })
}
