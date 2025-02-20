script;

// Test of unification of the Never type

impl [u32;0] {
    fn foo(self){
        log("32");
    }
}

impl [!;0] {
    fn foo(self){
        log("never");
    }
}

fn main() {
    let z:[u32;0] = [];

    // Should fail because z gets the type of its type ascription
    let y: [!;0] = z;
}
