contract;

enum Enum {
    C1 : (),
}

fn recover_on_path(e: Enum) -> Enum {
    match e {
        Enum:: => return e,
    }
}

fn recovery_witness() -> bool { 0 }
