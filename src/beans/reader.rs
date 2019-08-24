use std::io::Read;

pub struct Reader {
    content: String,
    pos: usize,
}

impl Reader {
    pub fn new<T: Read>(input: &mut T) -> Reader {
        let mut content = String::new();
        input.read_to_string(&mut content).unwrap();
        Reader {
            content: content,
            pos: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.content.chars().nth(self.pos + 1)
    }

    pub fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.content.chars().nth(self.pos)
    }

    pub fn prev(&self) -> Option<char> {
        self.content.chars().nth(self.pos - 1)
    }
}
