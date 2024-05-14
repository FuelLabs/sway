library;

use ::lib01::Lib01PublicStruct;
use ::lib01::lib01_nested::Lib01PublicNestedStruct;

use test_lib::lib01::Lib01PublicStruct as ExternalLib01PublicStruct;
use test_lib::lib01::lib01_nested::Lib01PublicNestedStruct as ExternalLib01PublicNestedStruct;

pub fn test_me() {
    let _ = Lib01PublicStruct { x: 0, y: 0 };
    let _ = Lib01PublicNestedStruct { x: 0, y: 0 };

    let _ = ExternalLib01PublicStruct { x: 0, y: 0 };
    let _ = ExternalLib01PublicNestedStruct { x: 0, y: 0 };
}