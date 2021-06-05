#[derive(Debug)]
pub struct CodeLine {
    pub text: String,
    pub is_string: bool,
    pub is_completed: bool,
    pub was_previously_stored: bool,
}

impl CodeLine {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_string: false,
            is_completed: false,
            was_previously_stored: false,
        }
    }

    pub fn default() -> Self {
        Self {
            text: "".into(),
            is_string: false,
            is_completed: false,
            was_previously_stored: false,
        }
    }

    pub fn empty_line() -> Self {
        Self {
            text: "".into(),
            is_string: false,
            is_completed: true,
            was_previously_stored: false,
        }
    }

    pub fn push_str(&mut self, line: &str) {
        self.text.push_str(line);
    }

    pub fn push_char(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
    }

    pub fn become_string(&mut self) {
        self.is_string = true;
    }

    pub fn end_string(&mut self) {
        self.is_string = false;
    }

    pub fn update_for_storage(&mut self, indentation: String) {
        self.was_previously_stored = true;
        self.text = format!("{}{}", indentation, self.text);
    }

    pub fn append_with_whitespace(&mut self, value: &str) {
        let last = self.text.chars().last();
        let is_previous_whitespace = if last.is_none() {
            true
        } else {
            last.unwrap() == ' '
        };

        if !is_previous_whitespace {
            self.push_char(' ');
        }

        self.push_str(value);
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}
