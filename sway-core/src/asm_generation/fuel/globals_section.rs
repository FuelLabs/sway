pub struct GlobalContent {
    pub name: String,
    pub size_in_bytes: usize,
    pub offset_in_bytes: usize,
}

pub struct Global(usize);

#[derive(Default)]
pub struct GlobalsSection {
    entries: Vec<GlobalContent>,
    current_offset_in_bytes: usize,
}

impl GlobalsSection {
    pub fn insert(&mut self, name: &str, size_in_bytes: usize) -> Global {
        let g = GlobalContent {
            name: name.to_string(),
            size_in_bytes,
            offset_in_bytes: self.current_offset_in_bytes,
        };
        self.entries.push(g);
        self.current_offset_in_bytes += size_in_bytes;
        Global(self.entries.len() - 1)
    }

    pub fn len_in_bytes(&self) -> usize {
        self.entries.iter().map(|x| x.size_in_bytes).sum()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&GlobalContent> {
        self.entries.iter().find(|x| x.name == name)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<GlobalContent> {
        self.entries.iter()
    }
}
