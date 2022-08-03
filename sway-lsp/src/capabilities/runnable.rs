#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RunnableType {
    /// This is the main_fn entry point for the predicate or script.
    MainFn,
    /// Place holder for when we have in language testing supported.
    /// The field holds the index of the test to run.
    _TestFn(u8),
}
