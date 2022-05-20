contract;

fn main() {
}

#[storage(read)]
fn can_read() {
  can_write();
}

#[storage(write)]
fn can_write() {}
