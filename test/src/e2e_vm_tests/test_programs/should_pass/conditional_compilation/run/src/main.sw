script;

#[cfg(target = "fuel")]
const VALUE: u64 = 40;
#[cfg(target = "evm")]
const VALUE: () = ();

#[cfg(target = "fuel")]
fn main() -> u64 {
  VALUE
}
#[cfg(target = "evm")]
fn main() {
  VALUE
}
