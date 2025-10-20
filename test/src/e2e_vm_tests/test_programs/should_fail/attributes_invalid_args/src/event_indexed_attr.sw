library;

#[event]
pub struct Ok {
    #[indexed]
    a: u32
}

#[event(arg = "")]
pub struct NotOk { }
