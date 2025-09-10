use std::iter::FusedIterator;

/// Position into an input stream
pub type Pos = usize;

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
            self.pos += c.len_utf8();
        }
        next
    }
}

impl<I> FusedIterator for Cursor<I> where I: FusedIterator<Item = char> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_cursor() {
        let mut c = Cursor::new("some string\na second line\n".chars());
        assert_eq!(c.pos(), Default::default());
        assert_eq!(c.next(), Some('s'));
        assert_eq!(c.next(), Some('o'));
        assert_eq!(c.next(), Some('m'));
        assert_eq!(c.next(), Some('e'));
        assert_eq!(c.pos(), 4);
        let string: String = c.by_ref().take(7).collect();
        assert_eq!(string, " string");

        assert_eq!(c.pos(), 11);
        assert_eq!(c.next(), Some('\n'));
        assert_eq!(c.pos(), 12);

        let cloned = c.clone();
        let cloned_pos = cloned.pos();

        let a_second_line: String = c.by_ref().take(13).collect();
        assert_eq!(a_second_line, "a second line");
        assert_eq!(c.pos(), 25);

        // checking independence of cloned cursor
        assert_eq!(cloned.pos(), cloned_pos);
        let a_second_line: String = cloned.take(13).collect();
        assert_eq!(a_second_line, "a second line");

        assert_eq!(c.pos(), 25);
        assert_eq!(c.next(), Some('\n'));
        assert_eq!(c.pos(), 26);
        assert_eq!(c.next(), None);
    }
}
