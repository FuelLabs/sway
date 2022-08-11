#[derive(Debug)]
pub struct CodeLine {
    pub text: String,
    pub is_completed: bool,
    pub was_previously_stored: bool,
    code_type: CodeType,
}

impl CodeLine {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_completed: false,
            was_previously_stored: false,
            code_type: CodeType::Default,
        }
    }

    pub fn default() -> Self {
        Self {
            text: "".into(),
            is_completed: false,
            was_previously_stored: false,
            code_type: CodeType::Default,
        }
    }

    pub fn empty_line() -> Self {
        Self {
            text: "".into(),
            is_completed: true,
            was_previously_stored: false,
            code_type: CodeType::Default,
        }
    }

    pub fn get_type(&self) -> CodeType {
        self.code_type
    }

    pub fn get_last_char(&self) -> Option<char> {
        self.text.chars().last()
    }

    pub fn is_string(&self) -> bool {
        self.code_type == CodeType::String
    }

    pub fn is_multiline_comment(&self) -> bool {
        self.code_type == CodeType::MultilineComment
    }

    pub fn become_string(&mut self) {
        self.code_type = CodeType::String
    }

    pub fn become_multiline_comment(&mut self) {
        self.code_type = CodeType::MultilineComment;
    }

    pub fn become_default(&mut self) {
        self.code_type = CodeType::Default;
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

    pub fn update_for_storage(&mut self, indentation: String) {
        self.was_previously_stored = true;
        self.text = format!("{}{}", indentation, self.text);
    }

    pub fn append_with_whitespace(&mut self, value: &str) {
        let last = self.text.chars().last();
        let is_previous_whitespace = Some(' ') == last;

        if !is_previous_whitespace && last != None {
            self.push_char(' ');
        }

        self.push_str(value);
    }

    pub fn append_equal_sign(&mut self) {
        let last = self.text.chars().last();

        match last {
            Some(c) if c == '!' || c == '<' || c == '>' => {
                self.push_str("= ");
            }
            _ => {
                self.append_with_whitespace("= ");
            }
        }
    }

    pub fn append_whitespace(&mut self) {
        let last = self.text.chars().last();

        match last {
            Some('(') => {} // do not add whitespace,
            _ => self.append_with_whitespace(""),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeType {
    String,
    Default,
    MultilineComment,
}
