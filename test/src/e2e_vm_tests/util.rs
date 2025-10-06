pub trait VecExt<T> {
    /// Retains the elements specified by the predicate `f`,
    /// and returns the elements that were removed.
    fn retain_and_get_removed<F>(&mut self, f: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool;
}

impl<T> VecExt<T> for Vec<T> {
    fn retain_and_get_removed<F>(&mut self, mut f: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut removed = Vec::new();
        let mut i = 0;
        while i < self.len() {
            if f(&mut self[i]) {
                i += 1;
            } else {
                let val = self.remove(i);
                removed.push(val);
            }
        }
        removed
    }
}

pub(crate) fn duration_to_str(duration: &std::time::Duration) -> String {
    let parts: [u16; 4] = [
        (duration.as_secs() / 3600) as u16,
        ((duration.as_secs() / 60) % 60) as u16,
        (duration.as_secs() % 60) as u16,
        (duration.as_millis() % 1000) as u16,
    ];

    // Hopefully we will never need to deal with hours :-)
    let parts = if parts[0] == 0 {
        &parts[1..]
    } else {
        &parts[..]
    };

    parts
        .iter()
        .map(|part| format!("{part:#02}"))
        .collect::<Vec<_>>()
        .join(":")
}
