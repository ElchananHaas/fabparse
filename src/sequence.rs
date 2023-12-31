/**
 * Trait for a sequence. This trait is implemented for slices and for str
 */
pub trait Sequence {
    type Item: Clone;
    /**
     * Try to split the sequence at an index. If this is out of range
     * this function will return None.
     */
    fn try_split_at<'a>(&'a self, mid: usize) -> Option<(&'a Self, &'a Self)>;
    /**
     * Try to split off the first element of a sequence.
     */
    fn try_split_front<'a>(&'a self) -> Option<(Self::Item, &'a Self)>;
    /**
     * Gets the length of a sequence. This will be in bytes for &str,
     * and number of elements for [T]
     */
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Sequence for [T] {
    type Item = T;

    fn try_split_at<'a>(&'a self, mid: usize) -> Option<(&'a Self, &'a Self)> {
        if mid > self.len() {
            None
        } else {
            Some(self.split_at(mid))
        }
    }

    fn try_split_front<'a>(&'a self) -> Option<(Self::Item, &'a Self)> {
        if self.len() == 0 {
            None
        } else {
            Some((self[0].clone(), &self[1..]))
        }
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl Sequence for str {
    type Item = char;

    fn try_split_at<'a>(&'a self, mid: usize) -> Option<(&'a Self, &'a Self)> {
        if mid > self.len() {
            None
        } else {
            Some(self.split_at(mid))
        }
    }

    fn try_split_front<'a>(&'a self) -> Option<(Self::Item, &'a Self)> {
        let res = self.chars().next();
        res.map(|char| (char, &self[char.len_utf8()..]))
    }
    fn len(&self) -> usize {
        self.len()
    }
}
