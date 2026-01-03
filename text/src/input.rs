type Pos = usize;

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

impl<I> std::iter::FusedIterator for Cursor<I> where I: std::iter::FusedIterator<Item = char> {}
