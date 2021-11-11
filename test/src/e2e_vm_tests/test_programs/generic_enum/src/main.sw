script;

fn main() {
    let x = Option::Some(10); 
    let y = Option::Some(true); 
}

enum Option<T> {
    Some: T,
    None: ()
}


