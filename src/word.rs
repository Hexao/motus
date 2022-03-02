// use std::slice::IterMut;

pub enum CharError {
    InvalidChar(char),
}

#[derive(Clone, Copy)]
pub struct Char(u8);

impl Char {
    pub fn used(&self) -> bool {
        (self.0 & 0b1000_0000) != 0
    }

    pub fn set_use(&mut self, used: bool) {
        if used {
            self.0 |= 0b1000_0000;
        } else {
            self.0 &= 0b0111_1111;
        }
    }
}

impl std::convert::TryFrom<char> for Char {
    type Error = CharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value >= 'a' && value <= 'z' {
            Ok(Char(value as u8 - 'a' as u8))
        } else {
            Err(CharError::InvalidChar(value))
        }
    }
}

impl std::convert::From<Char> for char {
    fn from(char: Char) -> Self {
        let char = char.0 & 0b0111_1111;
        (char + 'a' as u8) as char
    }
}

impl std::convert::From<&Char> for char {
    fn from(char: &Char) -> Self {
        let char = char.0 & 0b0111_1111;
        (char + 'a' as u8) as char
    }
}

impl std::fmt::Display for CharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharError::InvalidChar(c) => write!(f, "Can't convert '{}' into Char, it must be between 'a' and 'z'.", c),
        }
    }
}

pub struct Word {
    chars: Vec<Char>,
}

impl Word {
    pub fn new(data: String) -> Result<Self, CharError> {
        let mut chars = Vec::with_capacity(data.len());

        for c in data.chars() {
            chars.push(c.try_into()?);
        }

        Ok(Self { chars })
    }

    pub fn chars(&self) -> CharIte {
        CharIte { vec: &self.chars, index: 0 }
    }

    // pub fn chars_mut(&mut self) -> IterMut<Char> {
    //     self.chars.iter_mut()
    // }
}

impl std::fmt::Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = self.chars().collect();
        write!(f, "\"{}\"", string)
    }
}

pub struct CharIte<'a> {
    vec: &'a Vec<Char>,
    index: usize,
}

impl<'a> std::iter::Iterator for CharIte<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len() {
            let c = &self.vec[self.index];
            self.index += 1;
            Some(c.into())
        } else {
            None
        }
    }
}
