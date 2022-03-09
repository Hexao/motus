use std::fmt::Write;

#[derive(Clone)]
pub struct Mask {
    mask: Vec<LetterMask>,
    count: [u8; 26],
}

impl Mask {
    /// len doesn't count start char !
    pub fn new(start: char, len: u8) -> Self {
        let mut mask = Vec::with_capacity(len as usize + 1);
        mask.push(LetterMask::new(start));

        for _ in 0..len {
            mask.push(LetterMask::default());
        }

        Self { mask, count: [0; 26] }
    }

    pub fn match_with(&self, word: &str) -> Result<bool, MaskError> {
        if self.mask.len() == word.len() {
            let mask_match = self.mask
                .iter().zip(word.chars())
                .all(|(mask, char)| mask.match_with(char));

            if !mask_match {
                return Ok(false);
            }

            let mut count = [0; 26];
            for c in word.chars() {
                count[c as usize - 'a' as usize] += 1;
            }

            for (sc, c) in self.count.iter().zip(count) {
                let exact = sc & 0b1000_0000 != 0;
                let sc = sc & 0b0111_1111;

                if exact {
                    if sc != c {
                        return Ok(false);
                    }
                } else if sc > c {
                    return Ok(false);
                }
            }

            Ok(true)
        } else {
            Err(MaskError::IncompatibleLen("match_with"))
        }
    }

    pub fn update(&mut self, word: &str, result: &ResultState) -> Result<(), MaskError> {
        if self.mask.len() == word.len() && self.mask.len() == result.state.len() {
            // update count
            let mut stats = [(0, 0); 26];
            for (c, rc) in word.chars().zip(&result.state) {
                let index = c as usize - 'a' as usize;
                stats[index].0 += 1;

                if *rc != ResultColor::Blue {
                    stats[index].1 += 1;
                }
            }

            for (count, (all, ry)) in self.count.iter_mut().zip(stats) {
                if *count & 0b1000_0000 == 0 {
                    if all > ry {
                        *count = (ry & 0b0111_1111) + 0b1000_0000;
                    } else if *count < all {
                        *count = all;
                    }
                }
            }

            // update mask
            // red
            for ((i, c), rs) in word.char_indices().zip(&result.state) {
                if *rs == ResultColor::Red {
                    self.mask[i].set(c);
                }
            }

            // others
            let mut count = [0; 26];
            for c in self.mask.iter().filter_map(|lm| lm.red_char()) {
                count[c as usize - 'a' as usize] += 1;
            }

            for ((i, sc), c) in self.count.iter().enumerate().zip(count) {
                let char = (i as u8 + b'a') as char;
                let exact = sc & 0b1000_0000 != 0;
                let sc = sc & 0b0111_1111;

                if exact && sc == c {
                    for lm in self.mask.iter_mut().filter(|lm| lm.red_char().is_none()) {
                        lm.remove(char);
                    }
                } else {
                    word.char_indices().filter(|(_, c)| *c == char).for_each(|(i, _)| {
                        if self.mask[i].red_char().is_none() {
                            self.mask[i].remove(char);
                        }
                    })
                }
            }

            Ok(())
        } else {
            Err(MaskError::IncompatibleLen("update"))
        }
    }

    pub fn find_best(&self, dico: &[String]) -> Result<(usize, f32), MaskError> {
        let mut best_progress = (0, dico.len() as f32);
        let mut res = ResultState::new(self.mask.len());
        let mut self_clone = self.clone();
        // let mut targets_show = true;

        for (idx, word) in dico.iter().enumerate() {
            let mut matchs = 0.0;
            let mut sum = 0.0;

            for target in dico.iter() {
                if !self.match_with(target)? {
                    continue;
                }

                // if targets_show {
                //     println!("possibility: {}", target);
                // }

                res.update_with(word, target)?;
                self_clone.update(word, &res)?;
                let score = self_clone.filter(dico)?;

                if score > 1 || (score == 1 && res.complet()) {
                    sum += score as f32;
                    matchs += 1.0;
                }

                self_clone.revert_from(self)?;
            }

            // targets_show = false;
            let avg = sum / matchs;
            if avg < best_progress.1 {
                // println!("{} ({:>5.2})", word, avg);
                best_progress = (idx, avg);
            }
        }

        Ok(best_progress)
    }

    pub fn filter(&self, dico: &[String]) -> Result<usize, MaskError> {
        let mut count = 0;

        for word in dico {
            if self.match_with(word)? {
                count += 1;
            }
        }

        Ok(count)
    }

    fn revert_from(&mut self, rhs: &Mask) -> Result<(), MaskError> {
        if self.mask.len() == rhs.mask.len() {
            self.count = rhs.count;
            self.mask
                .iter_mut().zip(&rhs.mask)
                .for_each(|(lhs, rhs)| *lhs = *rhs);

            Ok(())
        } else {
            Err(MaskError::IncompatibleLen("revert_from"))
        }
    }
}

impl std::fmt::Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, lm) in self.mask.iter().enumerate() {
            write!(f, "{}:", i)?;

            for c in 'a'..='z' {
                if lm.match_with(c) {
                    f.write_char(c)?;
                } else {
                    f.write_char(' ')?;
                }
            }

            f.write_char('\n')?;
        }

        for (i, c) in self.count.iter().enumerate() {
            let exact = match (*c) & 0b1000_0000 {
                0b0000_0000 => '+',
                0b1000_0000 => '!',
                _ => unreachable!(),
            };
            let count = c & 0b0111_1111;
            let char = (i as u8 + b'a') as char;

            write!(f, "{}:{}{} ", char, count, exact)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum MaskError {
    IncompatibleLen(&'static str),
}

impl std::fmt::Display for MaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaskError::IncompatibleLen(fnc) => write!(f, "{}: Operand have incompatible length", fnc),
        }
    }
}

#[derive(Clone, Copy)]
struct LetterMask(u32);

impl LetterMask {
    fn new(char: char) -> Self {
        Self(Self::mask(char))
    }

    fn remove(&mut self, char: char) {
        self.0 &= u32::MAX ^ Self::mask(char);
    }

    fn set(&mut self, char: char) {
        self.0 = Self::mask(char);
    }

    fn match_with(&self, char: char) -> bool {
        self.0 & Self::mask(char) != 0
    }

    fn red_char(&self) -> Option<char> {
        if self.0.count_ones() == 1 {
            Some(('a' as u32 + self.0.trailing_zeros()) as u8 as char)
        } else {
            None
        }
    }

    #[inline(always)]
    fn mask(char: char) -> u32 {
        1 << (char as u8 - b'a')
    }
}

impl std::default::Default for LetterMask {
    fn default() -> Self {
        // 26 bits set ot 1
        Self(0x03_FF_FF_FF)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ResultState {
    state: Vec<ResultColor>,
}

impl ResultState {
    pub fn new(len: usize) -> Self {
        let mut state = vec![ResultColor::default(); len];
        state[0] = ResultColor::Red;
        Self { state }
    }

    pub fn update_with(&mut self, guess: &str, target: &str) -> Result<(), MaskError> {
        if self.state.len() == guess.len() && self.state.len() == target.len() {
            self.state.iter_mut().for_each(|rc| *rc = ResultColor::Blue);
            let mut used = 0_u16;

            // update red cells
            guess.char_indices().zip(target.chars()).for_each(|((i, g), t)| if g == t {
                self.state[i] = ResultColor::Red;
                used |= 1 << i;
            });

            // update possible yellow cells
            'guess: for (ig, g) in guess.char_indices() {
                'target: for (it, t) in target.char_indices() {
                    let mask = 1 << it;

                    if used & mask != 0 {
                        continue 'target;
                    }

                    if g == t {
                        self.state[ig] = ResultColor::Yellow;
                        used |= mask;

                        continue 'guess;
                    }
                }
            }

            Ok(())
        } else {
            Err(MaskError::IncompatibleLen("update_with"))
        }
    }

    pub fn complet(&self) -> bool {
        self.state.iter().all(|rc| *rc == ResultColor::Red)
    }
}

impl std::convert::TryFrom<&str> for ResultState {
    type Error = ConvertError;

    fn try_from(mask: &str) -> Result<Self, Self::Error> {
        let mut state = Vec::with_capacity(mask.len());

        for c in mask.chars() {
            state.push(c.try_into()?)
        }

        Ok(Self { state })
    }
}

impl std::fmt::Display for ResultState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = self.state.iter().map(|rc| -> char { (*rc).into() }).collect();
        write!(f, "{}", str)
    }
}

impl std::fmt::Debug for ResultState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResultState(\"{}\")", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResultColor {
    Blue,
    Yellow,
    Red,
}

impl std::default::Default for ResultColor {
    fn default() -> Self {
        Self::Blue
    }
}

impl std::convert::From<ResultColor> for char {
    fn from(rc: ResultColor) -> Self {
        match rc {
            ResultColor::Blue => 'b',
            ResultColor::Yellow => 'y',
            ResultColor::Red => 'r',
        }
    }
}

impl std::convert::TryFrom<char> for ResultColor {
    type Error = ConvertError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'r' | 'R' => Ok(ResultColor::Red),
            'y' | 'Y' => Ok(ResultColor::Yellow),
            'b' | 'B' => Ok(ResultColor::Blue),
            c => Err(ConvertError::InvalidChar(c)),
        }
    }
}

#[derive(Debug)]
pub enum ConvertError {
    InvalidChar(char),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::InvalidChar(c) => write!(f, "found invalid char in ResultStateMask: '{}'\n  avialable chars are: 'r', 'b' and 'y'", c),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_result_state() {
        use super::ResultState;

        let rs1 = ResultState::new(6);
        assert_eq!(rs1.to_string(), "rbbbbb");

        let rs2 = ResultState::new(9);
        assert_eq!(rs2.to_string(), "rbbbbbbbb");
    }

    #[test]
    fn compare_result_state() {
        use super::ResultState;

        let mut rs1 = ResultState::new(6);
        rs1.update_with("mourir", "manger").unwrap();
        assert_eq!(rs1.to_string(), "rbbbbr");

        // guess isn't actualy a real word, but it will be fine
        let mut rs2 = ResultState::new(8);
        rs2.update_with("mozozzgz", "montagne").unwrap();
        assert_eq!(rs2.to_string(), "rrbbbbyb");
    }

    #[test]
    fn build_result_state() {
        use super::ResultColor::*;
        use super::ResultState;

        let rs1: ResultState = "RBBYYR".try_into().unwrap();
        assert_eq!(rs1, ResultState {
            state: vec![Red, Blue, Blue, Yellow, Yellow, Red]
        });

        let rs2: ResultState = "rrbbbbyb".try_into().unwrap();
        assert_eq!(rs2, ResultState {
            state: vec![Red, Red, Blue, Blue, Blue, Blue, Yellow, Blue]
        });
    }

    #[test]
    fn red_char() {
        use super::LetterMask;

        let a = LetterMask(1 << 0);
        assert_eq!(a.red_char(), Some('a'));

        let m = LetterMask(1 << 12);
        assert_eq!(m.red_char(), Some('m'));

        let z = LetterMask(1 << 25);
        assert_eq!(z.red_char(), Some('z'));

        let oops = LetterMask((1 << 6) + (1 << 18));
        assert_eq!(oops.red_char(), None);
    }

    #[test]
    fn real_case() {
        use super::ResultColor::*;
        use super::ResultState;
        use super::Mask;

        let mut mask = Mask::new('i', 8);
        mask.update("inseminee", &ResultState {
            state: vec![Red, Red, Yellow, Red, Blue, Blue, Yellow, Red, Yellow],
        }).unwrap();

        println!("{:?}", mask);
    }
}