use crate::priv_prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    src: Arc<str>,
    start: usize,
    end: usize,
    path: Option<Arc<PathBuf>>,
}

impl Span {
    pub fn new(src: Arc<str>, start: usize, end: usize, path: Option<Arc<PathBuf>>) -> Option<Span> {
        if src.get(start..end).is_none() {
            return None;
        }
        Some(Span {
            src,
            start,
            end,
            path,
        })
    }
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
        assert_eq!(lhs.path, rhs.path);
        Span {
            src: lhs.src.clone(),
            start: lhs.start,
            end: rhs.end,
            path: lhs.path.clone(),
        }
    }

    pub fn src(&self) -> &Arc<str> {
        &self.src
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
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

