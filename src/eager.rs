use std::iter::{FusedIterator, TrustedLen};
use std::ops::Try;

/// Iterator that zips two iterators, checking that they have the same length during
/// construction.
#[derive(Debug, Clone)]
pub struct ZipEqEagerCheck<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

// SAFETY: a and b have the same length
impl<A: Iterator, B: Iterator> Iterator for ZipEqEagerCheck<A, B> {
    type Item = (A::Item, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.a.next().map(|a| (a, self.b.next().unwrap_unchecked())) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        super::size_hint_impl(self.a.size_hint(), self.b.size_hint())
    }

    fn count(self) -> usize {
        self.a.count()
    }

    fn last(self) -> Option<Self::Item> {
        unsafe { self.a.last().map(|a| (a, self.b.last().unwrap_unchecked())) }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe { self.a.nth(n).map(|a| (a, self.b.nth(n).unwrap_unchecked())) }
    }

    fn fold<I, F: FnMut(I, Self::Item) -> I>(self, init: I, mut f: F) -> I {
        let mut b = self.b;
        unsafe {
            self.a.fold(init, move |init, a| {
                f(init, (a, b.next().unwrap_unchecked()))
            })
        }
    }

    fn try_fold<I, F: FnMut(I, Self::Item) -> R, R>(&mut self, init: I, mut f: F) -> R
    where
        R: Try<Output = I>,
    {
        let b = &mut self.b;
        unsafe {
            self.a.try_fold(init, move |init: I, a: A::Item| {
                f(init, (a, b.next().unwrap_unchecked()))
            })
        }
    }
}

// SAFETY: a and b have the same length
impl<A: DoubleEndedIterator, B: DoubleEndedIterator> DoubleEndedIterator for ZipEqEagerCheck<A, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            self.a
                .next_back()
                .map(|a| (a, self.b.next_back().unwrap_unchecked()))
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            self.a
                .nth_back(n)
                .map(|a| (a, self.b.nth_back(n).unwrap_unchecked()))
        }
    }

    fn try_rfold<I, F, R>(&mut self, init: I, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(I, Self::Item) -> R,
        R: Try<Output = I>,
    {
        let b = &mut self.b;
        unsafe {
            self.a.try_rfold(init, move |init: I, a: A::Item| {
                f(init, (a, b.next_back().unwrap_unchecked()))
            })
        }
    }

    fn rfold<I, F>(self, init: I, mut f: F) -> I
    where
        Self: Sized,
        F: FnMut(I, Self::Item) -> I,
    {
        let mut b = self.b;
        unsafe {
            self.a.rfold(init, move |init, a| {
                f(init, (a, b.next_back().unwrap_unchecked()))
            })
        }
    }
}

impl<A: ExactSizeIterator, B: ExactSizeIterator> ExactSizeIterator for ZipEqEagerCheck<A, B> {
    fn len(&self) -> usize {
        self.a.len()
    }
}

unsafe impl<A: TrustedLen, B: Iterator> TrustedLen for ZipEqEagerCheck<A, B> {}
impl<A: FusedIterator, B: Iterator> FusedIterator for ZipEqEagerCheck<A, B> {}
