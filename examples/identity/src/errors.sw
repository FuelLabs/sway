library;

use core::codec::*;

pub enum MyError {
    UnauthorizedUser: Identity,
}
