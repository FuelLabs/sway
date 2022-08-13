#[derive(Clone, Debug)]
pub struct Punctuated<T, P> {
    pub value_separator_pairs: Vec<(T, P)>,
    pub final_value_opt: Option<Box<T>>,
}

impl<T, P> Punctuated<T, P> {
    pub fn empty() -> Self {
        Self {
            value_separator_pairs: vec![],
            final_value_opt: None,
        }
    }

    pub fn single(value: T) -> Self {
        Self {
            value_separator_pairs: vec![],
            final_value_opt: Some(Box::new(value)),
        }
    }
}

impl<T, P> IntoIterator for Punctuated<T, P> {
    type Item = T;
    type IntoIter = PunctuatedIter<T, P>;
    fn into_iter(self) -> PunctuatedIter<T, P> {
        PunctuatedIter {
            value_separator_pairs: self.value_separator_pairs.into_iter(),
            final_value_opt: self.final_value_opt,
        }
    }
}

pub struct PunctuatedIter<T, P> {
    value_separator_pairs: std::vec::IntoIter<(T, P)>,
    final_value_opt: Option<Box<T>>,
}

impl<T, P> Iterator for PunctuatedIter<T, P> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.value_separator_pairs.next() {
            Some((value, _separator)) => Some(value),
            None => self.final_value_opt.take().map(|final_value| *final_value),
        }
    }
}

impl<'a, T, P> IntoIterator for &'a Punctuated<T, P> {
    type Item = &'a T;
    type IntoIter = PunctuatedRefIter<'a, T, P>;
    fn into_iter(self) -> PunctuatedRefIter<'a, T, P> {
        PunctuatedRefIter {
            punctuated: self,
            index: 0,
        }
    }
}

pub struct PunctuatedRefIter<'a, T, P> {
    punctuated: &'a Punctuated<T, P>,
    index: usize,
}

impl<'a, T, P> Iterator for PunctuatedRefIter<'a, T, P> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.index > self.punctuated.value_separator_pairs.len() {
            return None;
        }
        match self.punctuated.value_separator_pairs.get(self.index) {
            None => match &self.punctuated.final_value_opt {
                Some(value) => {
                    self.index += 1;
                    Some(value)
                }
                None => None,
            },
            Some((value, _separator)) => {
                self.index += 1;
                Some(value)
            }
        }
    }
}
