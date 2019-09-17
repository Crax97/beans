#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reader() {
        let test = String::from("abcd");
        let mut r = Reader::new(test);

        assert!(r.prev() == None);
        assert!(r.peek().unwrap() == 'a');
        assert!(r.next().unwrap() == 'a');

        assert!(r.prev().unwrap() == 'a');
        assert!(r.peek().unwrap() == 'b');
        assert!(r.next().unwrap() == 'b');

        assert!(r.prev().unwrap() == 'b');
        assert!(r.peek().unwrap() == 'c');
        assert!(r.next().unwrap() == 'c');

        assert!(r.prev().unwrap() == 'c');
        assert!(r.peek().unwrap() == 'd');
        assert!(r.next().unwrap() == 'd');

        assert!(r.prev().unwrap() == 'd');
        assert!(r.peek() == None);
        assert!(r.next() == None);
    }
}

pub struct Reader {
    content: String,
    pos: usize,
}

impl Reader {
    pub fn new(input: String) -> Reader {
        Reader {
            content: input,
            pos: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.content.chars().nth(self.pos)
    }

    pub fn next(&mut self) -> Option<char> {
        let ch = self.content.chars().nth(self.pos);
        self.pos += 1;
        ch
    }

    pub fn prev(&self) -> Option<char> {
        if self.pos == 0 {
            return None;
        }
        self.content.chars().nth(self.pos - 1)
    }
}
