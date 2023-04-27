script;

enum MyNever {}

impl MyNever {
    fn into_any<T>(self) -> T {
        match self {}
    }
}

fn result_into_ok<T>(res: Result<T, MyNever>) -> T {
    match res {
        Ok(t) => t,
        // This branch can never be taken, and so the
        // compiler is happy to treat it as evaluating
        // to whatever type we wish - in this case, `T`.
        Err(never) => match never {},
    }
}

fn main() -> u64 {
    42
}
