script;

mod module0;
mod module1;

fn main() {
    let mut x = module0::Thing::new();
    let y = module1::Thing::new();
    x = y;
}
