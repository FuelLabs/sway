script;

mod bar;

// This test should fail to compile because the import statement below is missing
// use ::bar::Bar;

fn main() -> bool {
    let b = Bar {
        a: 5u32,
    };
    false
}
