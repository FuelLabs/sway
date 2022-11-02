library convert;

pub trait From {
    fn from(b: T) -> Self;
    fn into(self) -> T;
}
