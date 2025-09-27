library;

pub enum MyResult<T, E> {
    Ok: T,
    Err: E,
}

struct Data<T> {
    value: T,
    other: u64
}

struct Item<I> {
    item: I,
}

fn test<A, I>(arg: A) -> Item<I> {
    Item {
        item: arg,
    }
}

pub fn main() {
    example();
    let i: Item<u64> = test(true);
}

fn example() {
    let foo = MyResult::Ok::<Data<bool>, str[4]>(Data { value: true, other: 1 });
    foo.does_not_exist();
}
