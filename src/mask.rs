// use std::io::Write as flush;
use std::fmt::Write;
use rayon::iter::{
    IntoParallelRefIterator,
    IndexedParallelIterator,
    ParallelIterator
};

const A_USIZE: usize = b'a' as usize;

#[derive(Clone)]
pub struct Mask {
    mask: Vec<LetterMask>,
    count: [u8; 26],
}

impl Mask {
    /// len doesn't count start char !
    pub fn new(start: char, len: u8) -> Self {
        let mut mask = vec![LetterMask::default(); len as usize + 1];
        mask[0].set(start as u8);

        Self { mask, count: [0; 26] }
    }

    #[inline(always)]
    fn match_with(&self, word: &str) -> Result<bool, MaskError> {
        if self.mask.len() == word.len() {
            let mask_match = self.mask
                .iter().zip(word.as_bytes())
                .all(|(mask, &char)| mask.match_with(char));

            if !mask_match {
                return Ok(false);
            }

            let mut count = [0; 26];
            for &c in word.as_bytes() {
                count[c as usize - A_USIZE] += 1;
            }

            for (&sc, c) in self.count.iter().zip(count) {
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

    #[inline(always)]
    pub fn update(&mut self, word: &str, result: &ResultState) -> Result<(), MaskError> {
        if self.mask.len() == word.len() && self.mask.len() == result.state.len() {
            // update count
            let mut stats = [(0, 0); 26];
            for (&c, &rc) in word.as_bytes().iter().zip(&result.state) {
                let index = c as usize - A_USIZE;
                stats[index].0 += 1;

                if rc != ResultColor::Blue {
                    stats[index].1 += 1;
                }
            }

            for (count, (all, ry)) in self.count.iter_mut().zip(stats) {
                if *count & 0b1000_0000 == 0 {
                    if all > ry {
                        *count = ry | 0b1000_0000;
                    } else if *count < all {
                        *count = all;
                    }
                }
            }

            // update mask
            // red
            let iterator = word.as_bytes().iter()
                .enumerate()
                .zip(&result.state)
                .filter_map(|(data, &rs)|
                    if rs == ResultColor::Red { Some(data) } else { None }
                );

            for (i, &c) in iterator {
                self.mask[i].set(c);
            }

            // yellow
            let mut count = [0; 26];
            for c in self.mask.iter().filter_map(|lm| lm.red_char()) {
                count[c as usize - A_USIZE] += 1;
            }

            for ((i, &sc), c) in self.count.iter().enumerate().zip(count) {
                let char = i as u8 + b'a';
                let exact = sc & 0b1000_0000 != 0;
                let sc = sc & 0b0111_1111;

                if exact && sc == c {
                    for lm in &mut self.mask {
                        lm.remove(char);
                    }
                } else {
                    word.as_bytes().iter()
                        .zip(&mut self.mask)
                        .filter(|(&c, _)| c == char)
                        .for_each(|(_, mask)| mask.remove(char));
                }
            }

            Ok(())
        } else {
            Err(MaskError::IncompatibleLen("update"))
        }
    }

    #[inline(always)]
    pub fn find_best(&self, dico: &[String]) -> Result<(usize, f32), MaskError> {
        let mut valid_target = Vec::with_capacity(dico.len());

        // update valid target
        for target in dico.iter() {
            valid_target.push(self.match_with(target)?);
        }

        let best_progress = dico.par_iter().enumerate().fold(|| Ok((0, f32::MAX)), |best, (idx, word)| {
            let best_progress = match best {
                Ok(best) => best,
                _ => return best,
            };

            let mut res = ResultState::new(self.mask.len());
            let mut self_clone = self.clone();

            let mut states = [0; 3_usize.pow(8)];
            let mut matchs = 0.0;
            let mut sum = 0.0;

            let iterator = dico.iter().zip(&valid_target)
                .filter_map(|(target, &valid)| if valid { Some(target) } else { None });

            for target in iterator {
                res.update_with(word, target)?;
                let state_id = res.state_id();

                if states[state_id] > 0 {
                    sum += states[state_id] as f32;
                    matchs += 1.0;
                    continue;
                }

                self_clone.update(word, &res)?;

                match self_clone.filter(dico) {
                    FilterResult::Err(err) => return Err(err),
                    FilterResult::Count(score) => {
                        states[state_id] = score as u8;
                        sum += score as f32;
                        matchs += 1.0;
                    },
                    FilterResult::Word(_) => if res.complet() {
                        states[state_id] = 1;
                        matchs += 1.0;
                        sum += 1.0;
                    },
                }

                self_clone.revert_from(self);
            }

            let avg = sum / matchs;
            if avg < best_progress.1 {
                Ok((idx, avg))
            } else {
                Ok(best_progress)
            }

            // \x1B[1K clear the line \x1b[1G place the cursor in the first col
            // print!("\x1B[1K\rBest: {} ({:.2}) | current: {word} ({avg:.2})", dico[best_progress.0], best_progress.1);
            // std::io::stdout().flush().unwrap();
        }).reduce_with(|lhs, rhs| {
            match (lhs, rhs) {
                (Ok(lhs), Ok(rhs)) => {
                    if lhs.1 <= rhs.1 {
                        Ok(lhs)
                    } else {
                        Ok(rhs)
                    }
                }
                (Err(e), _) | (_, Err(e)) => Err(e)
            }
        }).unwrap();

        // println!();
        best_progress
    }

    #[inline(always)]
    pub fn filter<'a>(&self, dico: &'a[String]) -> FilterResult<'a> {
        let mut last_match = 0;
        let mut count = 0;

        for (id, word) in dico.iter().enumerate() {
            match self.match_with(word) {
                Err(err) => return FilterResult::Err(err),
                Ok(match_with) => if match_with {
                    last_match = id;
                    count += 1;
                }
            }
        }

        if count == 1 {
            FilterResult::Word(&dico[last_match])
        } else {
            FilterResult::Count(count)
        }
    }

    #[inline(always)]
    fn revert_from(&mut self, rhs: &Mask) {
        self.count = rhs.count;
        self.mask
            .iter_mut().zip(&rhs.mask)
            .for_each(|(lhs, rhs)| *lhs = *rhs);
    }
}

impl std::fmt::Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, lm) in self.mask.iter().enumerate() {
            write!(f, "{}:", i)?;

            for c in b'a'..=b'z' {
                if lm.match_with(c) {
                    f.write_char(c as char)?;
                } else {
                    f.write_char(' ')?;
                }
            }

            f.write_char('\n')?;
        }

        for (i, &c) in self.count.iter().enumerate() {
            let exact = match c & 0b1000_0000 {
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

pub enum FilterResult<'a> {
    Count(usize),
    Word(&'a str),
    Err(MaskError),
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
    #[inline(always)]
    fn remove(&mut self, char: u8) {
        if self.0.count_ones() > 1 {
            self.0 &= u32::MAX - Self::mask(char);
        }
    }

    #[inline(always)]
    fn set(&mut self, char: u8) {
        self.0 = Self::mask(char);
    }

    #[inline(always)]
    fn match_with(&self, char: u8) -> bool {
        self.0 & Self::mask(char) != 0
    }

    #[inline(always)]
    fn red_char(&self) -> Option<u8> {
        if self.0.count_ones() == 1 {
            Some(b'a' + self.0.trailing_zeros() as u8)
        } else {
            None
        }
    }

    #[inline(always)]
    fn mask(char: u8) -> u32 {
        1 << (char - b'a')
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
            guess.as_bytes()
                .iter()
                .enumerate()
                .zip(target.as_bytes())
                .for_each(|((i, &g), &t)| if g == t {
                    self.state[i] = ResultColor::Red;
                    used |= 1 << i;
                });

            // update possible yellow cells
            'guess: for (ig, &g) in guess.as_bytes().iter().enumerate() {
                'target: for (it, &t) in target.as_bytes().iter().enumerate() {
                    let mask = 1 << it;

                    if used & mask != 0 {
                        continue 'target;
                    }

                    if g == t && self.state[ig] == ResultColor::Blue {
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

    fn state_id(&self) -> usize {
        self.state.iter().fold(0, |state, rc| {
            state * 3 + match rc {
                ResultColor::Red => 0,
                ResultColor::Yellow => 1,
                ResultColor::Blue => 2,
            }
        })
    }

    pub fn complet(&self) -> bool {
        self.state.iter().all(|&rc| rc == ResultColor::Red)
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

        let mut rs2 = ResultState::new(7);
        rs2.update_with("marines", "manager").unwrap();
        assert_eq!(rs2.to_string(), "rrybyrb");

        // guess isn't actualy a real word, but it will be fine
        let mut rs3 = ResultState::new(8);
        rs3.update_with("mozozzgz", "montagne").unwrap();
        assert_eq!(rs3.to_string(), "rrbbbbyb");
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
        assert_eq!(a.red_char(), Some(b'a'));

        let m = LetterMask(1 << 12);
        assert_eq!(m.red_char(), Some(b'm'));

        let z = LetterMask(1 << 25);
        assert_eq!(z.red_char(), Some(b'z'));

        let oops = LetterMask((1 << 6) + (1 << 18));
        assert_eq!(oops.red_char(), None);
    }
}
