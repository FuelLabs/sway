contract;

trait A {
    #[storage(read)]
    fn f(self) -> bool;
} {
    fn g(self) -> bool {
        self.f()
    }
}
