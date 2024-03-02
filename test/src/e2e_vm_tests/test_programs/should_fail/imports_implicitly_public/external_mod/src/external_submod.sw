library;

pub enum ExtSubmodEnum {
    A: (),
    B: (),
}

pub fn external_submod_foo() -> u32 {
    let _ = external_submod_private();
    2
}

fn external_submod_private() -> ExtSubmodEnum {
    ExtSubmodEnum::A
}
