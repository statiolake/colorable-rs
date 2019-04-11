use std::collections::VecDeque;

pub struct Bytes {
    counter: usize,
    data: VecDeque<u8>,
}

impl Bytes {
    pub fn new() -> Bytes {
        Bytes {
            counter: 0,
            data: VecDeque::new(),
        }
    }

    pub fn read_count(&self) -> usize {
        self.counter
    }

    pub fn reset_counter(&mut self) {
        self.counter = 0;
    }

    pub fn unget(&mut self, b: u8) {
        if self.counter > 0 {
            self.counter -= 1;
        }
        self.data.push_front(b)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Iterator for Bytes {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.counter += 1;
        self.data.pop_front()
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
