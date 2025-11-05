library;

#[event]
#[event, event]
pub struct NotOk { }

#[event]
pub struct NotOk2 {
    #[indexed, indexed]
    a: u32
}
