use std::iter::FusedIterator;

/// Position into an input stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub index: usize,
    pub line: u32,
    pub column: u32,
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            index: 0,
            line: 1,
            column: 1,
        }
    }
}

/// A cursor over an input stream of characters.
/// It keeps track of the current position in the stream.
#[derive(Debug, Clone)]
pub struct Cursor<I> {
    // input iterator
    input: I,
    // current position in the stream
    pos: Pos,
}

impl<I> Cursor<I> {
    pub fn new(input: I) -> Self {
        Self {
            input,
            pos: Pos::default(),
        }
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }
}

impl<I> Cursor<I>
where
    I: Iterator<Item = char> + Clone,
{
    pub fn first(&self) -> Option<char> {
        self.input.clone().next()
    }
}

impl<I> Iterator for Cursor<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.input.next();
        if let Some(c) = next {
            self.pos.index += c.len_utf8();
            if c == '\n' {
                self.pos.line += 1;
                self.pos.column = 1;
            } else {
                self.pos.column += 1;
            }
        }
        next
    }
}

impl<I> FusedIterator for Cursor<I> where I: FusedIterator<Item = char> {}

#[cfg(test)]
mod tests {
    use super::{Cursor, Pos};

    #[test]
    fn test_input_cursor() {
        let mut c = Cursor::new("some string\na second line\n".chars());
        assert_eq!(c.pos(), Default::default());
        assert_eq!(c.next(), Some('s'));
        assert_eq!(c.next(), Some('o'));
        assert_eq!(c.next(), Some('m'));
        assert_eq!(c.next(), Some('e'));
        assert_eq!(
            c.pos(),
            Pos {
                index: 4,
                line: 1,
                column: 5
            }
        );
        let string: String = c.by_ref().take(7).collect();
        assert_eq!(string, " string");

        assert_eq!(
            c.pos(),
            Pos {
                index: 11,
                line: 1,
                column: 12
            }
        );
        assert_eq!(c.next(), Some('\n'));
        assert_eq!(
            c.pos(),
            Pos {
                index: 12,
                line: 2,
                column: 1
            }
        );

        let cloned = c.clone();
        let cloned_pos = cloned.pos();

        let a_second_line: String = c.by_ref().take(13).collect();
        assert_eq!(a_second_line, "a second line");
        assert_eq!(
            c.pos(),
            Pos {
                index: 25,
                line: 2,
                column: 14
            }
        );

        // checking independence of cloned cursor
        assert_eq!(cloned.pos(), cloned_pos);
        let a_second_line: String = cloned.take(13).collect();
        assert_eq!(a_second_line, "a second line");

        assert_eq!(
            c.pos(),
            Pos {
                index: 25,
                line: 2,
                column: 14
            }
        );
        assert_eq!(c.next(), Some('\n'));
        assert_eq!(
            c.pos(),
            Pos {
                index: 26,
                line: 3,
                column: 1
            }
        );
        assert_eq!(c.next(), None);
    }
}
