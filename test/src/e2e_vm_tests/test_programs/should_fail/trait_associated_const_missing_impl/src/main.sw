library;

trait ConstantId {
    const ID: u32;
}

struct Struct { }

impl ConstantId for Struct { }
