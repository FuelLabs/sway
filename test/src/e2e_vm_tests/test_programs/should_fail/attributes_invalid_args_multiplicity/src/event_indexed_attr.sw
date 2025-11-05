library;

#[event]
pub struct Ok1 {
    #[indexed]
    a: u32
}

#[event(arg = "")]
pub struct NotOk1 { }

#[event]
pub struct NotOk2 {
    #[indexed(arg = "")]
    a: u32
}
