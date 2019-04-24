use std::collections::VecDeque;

pub struct Bytes {
    unget: Vec<u8>,
    data: VecDeque<u8>,
}

impl Bytes {
    pub fn new() -> Bytes {
        Bytes {
            unget: Vec::new(),
            data: VecDeque::new(),
        }
    }

    pub fn read_count(&self) -> usize {
        self.unget.len()
    }

    pub fn reset_counter(&mut self) {
        self.unget.clear();
    }

    pub fn unget(&mut self, expect: u8) {
        if let Some(actual) = self.unget.pop() {
            assert_eq!(expect, actual);
            self.data.push_front(actual);
        } else {
            panic!("unget() called while no bytes had been read.");
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn exists(&self, b: u8) -> bool {
        self.data.iter().any(|&c| c == b)
    }
}

impl Iterator for Bytes {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.data.pop_front().map(|b| {
            self.unget.push(b);
            b
        })
    }
}

impl Extend<u8> for Bytes {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, data: I) {
        self.data.extend(data.into_iter());
    }
}

impl<'a> Extend<&'a u8> for Bytes {
    fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, data: I) {
        self.data.extend(data.into_iter());
    }
}
