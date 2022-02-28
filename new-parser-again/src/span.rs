use crate::priv_prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub(crate) src: Arc<str>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    /*
    pub fn new(src: Arc<str>) -> Span {
        let end = src.len();
        Span {
            src,
            start: 0,
            end,
        }
    }
    */

    pub fn as_str(&self) -> &str {
        &self.src[self.start..self.end]
    }

    pub fn join(lhs: &Span, rhs: &Span) -> Span {
        assert!(Arc::ptr_eq(&lhs.src, &rhs.src));
        Span {
            src: lhs.src.clone(),
            start: lhs.start,
            end: rhs.end,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt
        .debug_struct("Span")
        .field("start", &self.start)
        .field("end", &self.end)
        .field("as_str", &self.as_str())
        .finish()
    }
}

