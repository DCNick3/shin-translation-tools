//! This module contains an implementation of UAX#14 based on `xi-unicode` crate (https://github.com/xi-editor/xi-editor/tree/a2dea3059312795c77caadc639df49bf8a7008eb/rust/unicode)

use crate::layout::linebreak::tables::{
    LINEBREAK_1_2, LINEBREAK_3_CHILD, LINEBREAK_3_ROOT, LINEBREAK_4_LEAVES, LINEBREAK_4_MID,
    LINEBREAK_4_ROOT, LINEBREAK_STATE_MACHINE, N_LINEBREAK_CATEGORIES,
};

mod tables;

/// The Unicode line breaking property of the given code point.
///
/// This is given as a numeric value which matches the ULineBreak
/// enum value from ICU.
pub fn linebreak_property(cp: char) -> u8 {
    let cp = cp as usize;
    if cp < 0x800 {
        LINEBREAK_1_2[cp]
    } else if cp < 0x10000 {
        let child = LINEBREAK_3_ROOT[cp >> 6];
        LINEBREAK_3_CHILD[(child as usize) * 0x40 + (cp & 0x3f)]
    } else {
        let mid = LINEBREAK_4_ROOT[cp >> 12];
        let leaf = LINEBREAK_4_MID[(mid as usize) * 0x40 + ((cp >> 6) & 0x3f)];
        LINEBREAK_4_LEAVES[(leaf as usize) * 0x40 + (cp & 0x3f)]
    }
}

#[derive(Clone)]
struct CharIterator<'a> {
    inner: std::slice::Iter<'a, super::layouter::Command<'a>>,
    index: usize,
}

impl<'a> Iterator for CharIterator<'a> {
    type Item = (usize, super::layouter::Char);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => break None,
                Some(super::layouter::Command::Unparsed(_)) => {
                    self.index += 1;
                    continue;
                }
                Some(&super::layouter::Command::Char(c)) => {
                    let result = (self.index, c);
                    self.index += 1;
                    break Some(result);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct LineBreakIterator<'a> {
    char_iter: CharIterator<'a>,
    state: u8,
}

impl<'a> LineBreakIterator<'a> {
    /// Create a new iterator for the given string slice.
    pub fn new(s: &'a [super::layouter::Command<'a>]) -> LineBreakIterator<'a> {
        let mut char_iter = CharIterator {
            inner: s.iter(),
            index: 0,
        };

        if let Some((_, c)) = char_iter.next() {
            let lb = linebreak_property(c.codepoint);
            LineBreakIterator {
                char_iter,
                state: lb,
            }
        } else {
            LineBreakIterator {
                char_iter,
                state: 0,
            }
        }
    }
}

impl<'a> Iterator for LineBreakIterator<'a> {
    type Item = (usize, bool);

    // return break pos and whether it's a hard break
    fn next(&mut self) -> Option<(usize, bool)> {
        loop {
            match self.char_iter.next() {
                None => break None,
                Some((index, c)) => {
                    let lb = linebreak_property(c.codepoint);
                    let i = (self.state as usize) * N_LINEBREAK_CATEGORIES + (lb as usize);
                    let new = LINEBREAK_STATE_MACHINE[i];
                    //println!("{:?}[{}], state {} + lb {} -> {}", &self.s[self.ix..], self.ix, self.state, lb, new);

                    if (new as i8) < 0 {
                        // break found
                        self.state = new & 0x3f;
                        if !c.has_furigana {
                            // characters having furigana can't be broken up
                            // ideally we should support breaking at furigana start
                            // but this is currently buggy because we have to insert a break command in right place (before @< or after @>)
                            // currently we don't look at commands at all, so we can't do that
                            // this currently inserts breaks right _before_ a character, so we only handle breaking at furigana end
                            return Some((index, new >= 0xc0));
                        }
                    } else {
                        self.state = new;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Write;

    use unicode_width::UnicodeWidthChar;

    use crate::layout::layouter::{Char, Command};

    #[test]
    pub fn smoke() {
        let paragraph = [
            Command::Char(Char {
                codepoint: 'H',
                has_furigana: false,
                pos_x: 0.0,
                scale: 1.3333334,
                width: 32.0,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'e',
                has_furigana: false,
                pos_x: 32.0,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'l',
                has_furigana: false,
                pos_x: 57.333336,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'l',
                has_furigana: false,
                pos_x: 68.0,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'o',
                has_furigana: false,
                pos_x: 78.666664,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 105.33333,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'w',
                has_furigana: false,
                pos_x: 118.666664,
                scale: 1.3333334,
                width: 33.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'o',
                has_furigana: false,
                pos_x: 152.0,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'r',
                has_furigana: false,
                pos_x: 178.66667,
                scale: 1.3333334,
                width: 18.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'l',
                has_furigana: false,
                pos_x: 197.33334,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'd',
                has_furigana: false,
                pos_x: 208.00002,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: '!',
                has_furigana: false,
                pos_x: 234.66669,
                scale: 1.3333334,
                width: 9.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 244.00002,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'I',
                has_furigana: false,
                pos_x: 257.33334,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 268.0,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'a',
                has_furigana: false,
                pos_x: 281.33334,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'm',
                has_furigana: false,
                pos_x: 306.6667,
                scale: 1.3333334,
                width: 37.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 344.00003,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'a',
                has_furigana: false,
                pos_x: 357.33337,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 382.66672,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 's',
                has_furigana: false,
                pos_x: 396.00006,
                scale: 1.3333334,
                width: 24.0,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 't',
                has_furigana: false,
                pos_x: 420.00006,
                scale: 1.3333334,
                width: 18.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'r',
                has_furigana: false,
                pos_x: 438.66672,
                scale: 1.3333334,
                width: 18.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'i',
                has_furigana: false,
                pos_x: 457.33337,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'n',
                has_furigana: false,
                pos_x: 468.00003,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'g',
                has_furigana: false,
                pos_x: 494.6667,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ',',
                has_furigana: false,
                pos_x: 521.3334,
                scale: 1.3333334,
                width: 9.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 530.6667,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'w',
                has_furigana: false,
                pos_x: 544.0,
                scale: 1.3333334,
                width: 33.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'i',
                has_furigana: false,
                pos_x: 577.3333,
                scale: 1.3333334,
                width: 10.666667,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 't',
                has_furigana: false,
                pos_x: 588.0,
                scale: 1.3333334,
                width: 18.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'h',
                has_furigana: false,
                pos_x: 606.6667,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 633.3334,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 's',
                has_furigana: false,
                pos_x: 646.6667,
                scale: 1.3333334,
                width: 24.0,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'o',
                has_furigana: false,
                pos_x: 670.6667,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'm',
                has_furigana: false,
                pos_x: 697.3334,
                scale: 1.3333334,
                width: 37.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'e',
                has_furigana: false,
                pos_x: 734.6667,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: ' ',
                has_furigana: false,
                pos_x: 760.0,
                scale: 1.3333334,
                width: 13.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'c',
                has_furigana: false,
                pos_x: 773.3333,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'o',
                has_furigana: false,
                pos_x: 798.6666,
                scale: 1.3333334,
                width: 26.666668,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'm',
                has_furigana: false,
                pos_x: 825.3333,
                scale: 1.3333334,
                width: 37.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'm',
                has_furigana: false,
                pos_x: 862.6666,
                scale: 1.3333334,
                width: 37.333336,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 'a',
                has_furigana: false,
                pos_x: 899.99994,
                scale: 1.3333334,
                width: 25.333334,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: 's',
                has_furigana: false,
                pos_x: 925.33325,
                scale: 1.3333334,
                width: 24.0,
                height: 37.333336,
            }),
            Command::Char(Char {
                codepoint: '.',
                has_furigana: false,
                pos_x: 949.33325,
                scale: 1.3333334,
                width: 9.333334,
                height: 37.333336,
            }),
        ];

        let mut breaks = super::LineBreakIterator::new(&paragraph).peekable();

        let mut data = String::new();
        let mut annot = String::new();

        for (i, c) in (0..).zip(&paragraph) {
            let is_break = if let Some(&(break_index, _is_hard)) = breaks.peek() {
                // TODO: I think we should ignore hard breaks. They are not supposed to be present in the SNR
                if i >= break_index {
                    breaks.next();
                }

                break_index == i
            } else {
                false
            };

            let mut width = 1;
            if let Command::Char(c) = c {
                width = c.codepoint.width().unwrap();
                write!(data, " {}", c.codepoint).unwrap();
            } else {
                write!(data, " _").unwrap();
            }

            let c = if is_break { 'X' } else { '-' };
            let c = std::iter::repeat_n(c, width).collect::<String>();
            write!(annot, "{} ", c).unwrap();
        }
        println!("{data}");
        println!("{annot}");
    }
}
