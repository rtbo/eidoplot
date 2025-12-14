use std::iter::FusedIterator;

#[derive(Debug)]
pub struct Boundaries {
    start: usize,
    end: usize,
    vec: Vec<usize>,
}

impl Boundaries {
    pub fn new(start: usize, end: usize) -> Boundaries {
        Boundaries {
            start,
            end,
            vec: Vec::new(),
        }
    }
    pub fn check_in(&mut self, i: usize) {
        if self.start < i && i < self.end {
            self.vec.push(i);
        }
    }
    pub fn into_iter(mut self) -> BoundariesIter {
        self.vec.sort_unstable();
        self.vec.dedup();
        BoundariesIter {
            start: self.start,
            end: self.end,
            i: 0,
            vec: self.vec,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundariesIter {
    start: usize,
    end: usize,
    i: usize,
    vec: Vec<usize>,
}

impl Iterator for BoundariesIter {
    type Item = (usize, usize);
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            let start = self.start;
            let end = if self.i < self.vec.len() {
                let i = self.i;
                self.i += 1;
                self.vec[i]
            } else {
                self.end
            };
            self.start = end;
            Some((start, end))
        }
    }
}

impl ExactSizeIterator for BoundariesIter {
    fn len(&self) -> usize {
        if self.start == self.end {
            0
        } else {
            self.vec.len() + 1
        }
    }
}

impl FusedIterator for BoundariesIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundaries() {
        let mut b = Boundaries::new(1, 10);
        b.check_in(0);
        b.check_in(3);
        b.check_in(6);
        b.check_in(12);
        let mut b = b.into_iter();
        assert_eq!(b.len(), 3);
        assert_eq!(b.next(), Some((1, 3)));
        assert_eq!(b.next(), Some((3, 6)));
        assert_eq!(b.next(), Some((6, 10)));
        assert_eq!(b.next(), None);
    }
}
