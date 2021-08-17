#[macro_export]
macro_rules! map {
    ($( $key: expr => $val: expr ),* $(,)*) => {{
        let mut map = ::std::collections::HashMap::default();
        $( map.insert($key, $val); )*
        map
    }}
}
