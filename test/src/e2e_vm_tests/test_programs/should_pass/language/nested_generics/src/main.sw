script;

enum MyOption<T> {
    Some: T,
    None: (),
}

impl<T> MyOption<T> {
    fn new() -> Self {
        Self::None
    }

    fn is_none(self) -> bool {
        true
    }
}

fn generic_arg_in_function_method_call<T>() {
    let o: MyOption<u64> = MyOption::None;
    let _ = o.is_none();
    
    let o: MyOption<MyOption<u64>> = MyOption::None;
    let _ = o.is_none();

    let o: MyOption<T> = MyOption::None;
    let _ = o.is_none();
    
    let o: MyOption<MyOption<T>> = MyOption::None;
    let _ = o.is_none();
    
    let _ = MyOption::is_none(o);
}

fn generic_arg_in_function_associated_function_call<T>() {
    let _ = MyOption::<u64>::new();
    let o: MyOption<u64> = MyOption::new();
    
    let _ = MyOption::<MyOption<u64>>::new();
    let o: MyOption<MyOption<u64>> = MyOption::new();

    let _ = MyOption::<T>::new();
    let o: MyOption<T> = MyOption::new();
    
    let _ = MyOption::<MyOption<T>>::new();
}

struct S<T> { }

impl<T> S<T> {
    fn generic_arg_in_type() {
        let o: MyOption<u64> = MyOption::None;
        let _ = o.is_none();
        
        let o: MyOption<MyOption<u64>> = MyOption::None;
        let _ = o.is_none();
    
        let o: MyOption<T> = MyOption::None;
        let _ = o.is_none();
        
        let o: MyOption<MyOption<T>> = MyOption::None;
        let _ = o.is_none();
        
        let _ = MyOption::is_none(o);
    }
}

pub fn main() ->  bool {
    generic_arg_in_function_method_call::<(())>();
    S::<()>::generic_arg_in_type();

    generic_arg_in_function_associated_function_call::<()>();

    true
}