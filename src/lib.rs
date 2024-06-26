use core::cmp::Ordering;
use core::hash::{BuildHasher, Hash};
use std::collections::VecDeque;
use strength_reduce::StrengthReducedU16;

/// Default hasher for [`MinimizerQueue`] and [`ImplicitMinimizerQueue`].
pub type DefaultHashBuilder = wyhash2::WyHash;

/// A monotone queue that can compute consecutive minimizers in constant time.
///
/// # Examples
///
/// ```
/// use minimizer_queue::MinimizerQueue;
///
/// let mut queue = MinimizerQueue::new(3); // width 3
/// queue.insert(1);
/// queue.insert(2);
/// queue.insert(3);
/// queue.get_min(); // element with the smallest hash among 1, 2 and 3
///
/// queue.insert(4);
/// queue.get_min(); // element with the smallest hash among 2, 3 and 4
/// ```
pub struct MinimizerQueue<T: Hash + Copy, S: BuildHasher = DefaultHashBuilder> {
    deq: VecDeque<(T, u64, u16)>,
    width: StrengthReducedU16,
    hash_builder: S,
    pos: u16,
}

impl<T: Hash + Copy> MinimizerQueue<T> {
    /// Creates an empty `MinimizerQueue` with the given width.
    #[inline]
    pub fn new(width: u16) -> Self {
        Self::with_seed(width, width as u64)
    }

    /// Creates an empty `MinimizerQueue` with the given width and seed.
    /// Changing the seed will change the ordering of the minimizers.
    #[inline]
    pub fn with_seed(width: u16, seed: u64) -> Self {
        Self::with_hasher(width, DefaultHashBuilder::with_seed(seed))
    }
}

impl<T: Hash + Copy, S: BuildHasher> MinimizerQueue<T, S> {
    /// Creates an empty `MinimizerQueue` with the given width and hasher.
    /// The hasher will define the ordering of the minimizers, based on their hashes.
    pub fn with_hasher(width: u16, hash_builder: S) -> Self {
        Self {
            deq: VecDeque::with_capacity(width as usize),
            width: StrengthReducedU16::new(width),
            hash_builder,
            pos: 0,
        }
    }

    /// Returns the width of the `MinimizerQueue`.
    #[inline]
    pub fn width(&self) -> usize {
        self.width.get() as usize
    }

    /// Returns `true` if the `MinimizerQueue` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.deq.is_empty()
    }

    /// Returns `true` if there are multiple minimizers in the queue.
    #[inline]
    pub fn multiple_mins(&self) -> bool {
        self.deq.len() >= 2 && self.deq[0].1 == self.deq[1].1
    }

    /// Returns the leftmost minimizer in the queue.
    #[inline]
    pub fn get_min(&self) -> T {
        debug_assert!(!self.deq.is_empty(), "MinimizerQueue is empty");
        self.deq[0].0
    }

    /// Returns the leftmost minimizer and its relative position in the queue.
    #[inline]
    pub fn get_min_pos(&self) -> (T, usize) {
        debug_assert!(!self.deq.is_empty(), "MinimizerQueue is empty");
        let (x, _, pos) = self.deq[0];
        let rel_pos = ((self.width.get() - self.pos + pos) % self.width) as usize;
        (x, rel_pos)
    }

    /// Returns the innermost minimizer and its relative position in the queue, with a second choice in case of tie.
    #[inline]
    pub fn get_inner_min_pos(&self) -> (T, usize, Option<(T, usize)>) {
        debug_assert!(!self.deq.is_empty(), "MinimizerQueue is empty");
        let width = self.width.get();
        let start = width - self.pos;
        let (mut x, hash, x_pos) = self.deq[0];
        let mut x_pos = (start + x_pos) % self.width;
        let mut i = 1;
        while i < self.deq.len() && self.deq[i].1 == hash {
            let (y, _, y_pos) = self.deq[i];
            let y_pos = (start + y_pos) % self.width;
            match x_pos.cmp(&(width - 1 - y_pos)) {
                Ordering::Less => {
                    x = y;
                    x_pos = y_pos;
                }
                Ordering::Equal => return (x, x_pos as usize, Some((y, y_pos as usize))),
                Ordering::Greater => return (x, x_pos as usize, None),
            }
            i += 1;
        }
        (x, x_pos as usize, None)
    }

    /// Inserts `x` in the queue and updates the current minimizer.
    #[inline]
    pub fn insert(&mut self, x: T) {
        self.insert_with_hash(x, self.hash_builder.hash_one(x))
    }

    /// Inserts `x` in the queue with the given hash and updates the current minimizer.
    pub fn insert_with_hash(&mut self, x: T, hash: u64) {
        if !self.deq.is_empty() && self.deq[0].2 == self.pos {
            self.deq.pop_front();
        }
        let mut i = self.deq.len();
        while i > 0 && hash < self.deq[i - 1].1 {
            i -= 1;
        }
        self.deq.truncate(i);
        self.deq.push_back((x, hash, self.pos));
        self.pos = (self.pos + 1) % self.width;
    }

    /// Clears the deque, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.deq.clear()
    }
}

/// A monotone queue that can compute the positions of consecutive minimizers in constant time.
///
/// # Examples
///
/// ```
/// use minimizer_queue::ImplicitMinimizerQueue;
///
/// let mut queue = ImplicitMinimizerQueue::new(3); // width 3
/// queue.insert(&1);
/// queue.insert(&2);
/// queue.insert(&3);
/// queue.get_min_pos(); // position of the element with the smallest hash among 1, 2 and 3
///
/// queue.insert(&4);
/// queue.get_min_pos(); // position of the element with the smallest hash among 2, 3 and 4
/// ```
pub struct ImplicitMinimizerQueue<S: BuildHasher = DefaultHashBuilder> {
    deq: VecDeque<(u64, u16)>,
    width: StrengthReducedU16,
    hash_builder: S,
    pos: u16,
}

impl ImplicitMinimizerQueue {
    /// Creates an empty `ImplicitMinimizerQueue` with the given width.
    #[inline]
    pub fn new(width: u16) -> Self {
        Self::with_seed(width, width as u64)
    }

    /// Creates an empty `ImplicitMinimizerQueue` with the given width and seed.
    /// Changing the seed will change the ordering of the minimizers.
    #[inline]
    pub fn with_seed(width: u16, seed: u64) -> Self {
        Self::with_hasher(width, DefaultHashBuilder::with_seed(seed))
    }
}

impl<S: BuildHasher> ImplicitMinimizerQueue<S> {
    /// Creates an empty `ImplicitMinimizerQueue` with the given width and hasher.
    /// The hasher will define the ordering of the minimizers, based on their hashes.
    pub fn with_hasher(width: u16, hash_builder: S) -> Self {
        Self {
            deq: VecDeque::with_capacity(width as usize),
            width: StrengthReducedU16::new(width),
            hash_builder,
            pos: 0,
        }
    }

    /// Returns the width of the `ImplicitMinimizerQueue`.
    #[inline]
    pub fn width(&self) -> usize {
        self.width.get() as usize
    }

    /// Returns `true` if the `ImplicitMinimizerQueue` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.deq.is_empty()
    }

    /// Returns `true` if there are multiple minimizers in the queue.
    #[inline]
    pub fn multiple_mins(&self) -> bool {
        self.deq.len() >= 2 && self.deq[0].0 == self.deq[1].0
    }

    /// Returns the relative position of the leftmost minimizer.
    #[inline]
    pub fn get_min_pos(&self) -> usize {
        debug_assert!(!self.deq.is_empty(), "ImplicitMinimizerQueue is empty");
        let (_, pos) = self.deq[0];
        ((self.width.get() - self.pos + pos) % self.width) as usize
    }

    /// Returns the relative position of the innermost minimizer, with a second choice in case of tie.
    #[inline]
    pub fn get_inner_min_pos(&self) -> (usize, Option<usize>) {
        debug_assert!(!self.deq.is_empty(), "MinimizerQueue is empty");
        let width = self.width.get();
        let start = width - self.pos;
        let (hash, x_pos) = self.deq[0];
        let mut x_pos = (start + x_pos) % self.width;
        let mut i = 1;
        while i < self.deq.len() && self.deq[i].0 == hash {
            let (_, y_pos) = self.deq[i];
            let y_pos = (start + y_pos) % self.width;
            match x_pos.cmp(&(width - 1 - y_pos)) {
                Ordering::Less => {
                    x_pos = y_pos;
                }
                Ordering::Equal => return (x_pos as usize, Some(y_pos as usize)),
                Ordering::Greater => return (x_pos as usize, None),
            }
            i += 1;
        }
        (x_pos as usize, None)
    }

    /// Inserts `x` in the queue and updates the current minimizer.
    #[inline]
    pub fn insert<T: Hash>(&mut self, x: &T) {
        self.insert_hash(self.hash_builder.hash_one(x))
    }

    /// Inserts `x` in the queue with the given hash and updates the current minimizer.
    pub fn insert_hash(&mut self, hash: u64) {
        if !self.deq.is_empty() && self.deq[0].1 == self.pos {
            self.deq.pop_front();
        }
        let mut i = self.deq.len();
        while i > 0 && hash < self.deq[i - 1].0 {
            i -= 1;
        }
        self.deq.truncate(i);
        self.deq.push_back((hash, self.pos));
        self.pos = (self.pos + 1) % self.width;
    }

    /// Clears the deque, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.deq.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nohash_hasher::BuildNoHashHasher;

    #[test]
    fn test_get_min() {
        let mut queue = MinimizerQueue::with_hasher(3, BuildNoHashHasher::<usize>::default());

        let vals = [1usize, 2, 3, 0, 7, 8, 9, 100, 3, 4, 7, 8];
        let mut mins = Vec::with_capacity(vals.len() - queue.width() + 1);

        for &val in vals.iter().take(queue.width() - 1) {
            queue.insert(val);
        }
        for &val in vals.iter().skip(queue.width() - 1) {
            queue.insert(val);
            mins.push(queue.get_min());
        }

        assert_eq!(mins, vec![1, 0, 0, 0, 7, 8, 3, 3, 3, 4]);
    }

    #[test]
    fn test_get_min_pos() {
        let mut queue = MinimizerQueue::with_hasher(3, BuildNoHashHasher::<usize>::default());

        let vals = [1usize, 2, 3, 0, 7, 8, 9, 100, 3, 4, 7, 8];
        let mut mins_pos = Vec::with_capacity(vals.len() - queue.width() + 1);

        for &val in vals.iter().take(queue.width() - 1) {
            queue.insert(val);
        }
        for &val in vals.iter().skip(queue.width() - 1) {
            queue.insert(val);
            mins_pos.push(queue.get_min_pos());
        }

        assert_eq!(
            mins_pos,
            vec![
                (1, 0),
                (0, 2),
                (0, 1),
                (0, 0),
                (7, 0),
                (8, 0),
                (3, 2),
                (3, 1),
                (3, 0),
                (4, 0)
            ]
        );
    }

    #[test]
    fn test_implicit_get_min_pos() {
        let mut queue =
            ImplicitMinimizerQueue::with_hasher(3, BuildNoHashHasher::<usize>::default());

        let vals = [1usize, 2, 3, 0, 7, 8, 9, 100, 3, 4, 7, 8];
        let mut mins_pos = Vec::with_capacity(vals.len() - queue.width() + 1);

        for val in vals.iter().take(queue.width() - 1) {
            queue.insert(val);
        }
        for val in vals.iter().skip(queue.width() - 1) {
            queue.insert(val);
            mins_pos.push(queue.get_min_pos());
        }

        assert_eq!(mins_pos, vec![0, 2, 1, 0, 0, 0, 2, 1, 0, 0]);
    }

    #[test]
    fn test_get_inner_min_pos() {
        let mut queue = MinimizerQueue::with_hasher(3, BuildNoHashHasher::<usize>::default());

        let vals = [1usize, 2, 3, 2, 2, 3, 1];
        let mut inner_mins_pos = Vec::with_capacity(vals.len() - queue.width() + 1);

        for &val in vals.iter().take(queue.width() - 1) {
            queue.insert(val);
        }
        for &val in vals.iter().skip(queue.width() - 1) {
            queue.insert(val);
            inner_mins_pos.push(queue.get_inner_min_pos());
        }

        assert_eq!(
            inner_mins_pos,
            vec![
                (1, 0, None),
                (2, 0, Some((2, 2))),
                (2, 1, None),
                (2, 1, None),
                (1, 2, None),
            ]
        );
    }

    #[test]
    fn test_implicit_get_inner_min_pos() {
        let mut queue =
            ImplicitMinimizerQueue::with_hasher(3, BuildNoHashHasher::<usize>::default());

        let vals = [1usize, 2, 3, 2, 2, 3, 1];
        let mut inner_mins_pos = Vec::with_capacity(vals.len() - queue.width() + 1);

        for val in vals.iter().take(queue.width() - 1) {
            queue.insert(val);
        }
        for val in vals.iter().skip(queue.width() - 1) {
            queue.insert(val);
            inner_mins_pos.push(queue.get_inner_min_pos());
        }

        assert_eq!(
            inner_mins_pos,
            vec![(0, None), (0, Some(2)), (1, None), (1, None), (2, None),]
        );
    }
}
