use core::iter::{FusedIterator, TrustedLen};
use core::ops::Try;

/// Iterator that zips two iterators, checking that they have the same length during
/// construction.
#[derive(Debug, Clone)]
pub struct ZipEqEagerCheck<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

#[inline]
unsafe fn unreachable_unchecked() -> ! {
    #[cfg(not(debug_assertions))]
    unsafe {
        core::hint::unreachable_unchecked()
    }
    #[cfg(debug_assertions)]
    unreachable!()
}

#[inline]
unsafe fn both_or_none<T, U>(t: Option<T>, u: Option<U>) -> Option<(T, U)> {
    match (t, u) {
        (Some(a), Some(b)) => Some((a, b)),
        (None, None) => None,
        _ => unsafe { unreachable_unchecked() },
    }
}

// SAFETY: a and b have the same length
impl<A: Iterator, B: Iterator> Iterator for ZipEqEagerCheck<A, B> {
    type Item = (A::Item, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { both_or_none(self.a.next(), self.b.next()) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        super::size_hint_impl(self.a.size_hint(), self.b.size_hint())
    }

    fn last(self) -> Option<Self::Item> {
        unsafe { both_or_none(self.a.last(), self.b.last()) }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe { both_or_none(self.a.nth(n), self.b.nth(n)) }
    }

    #[inline]
    fn fold<I, F: FnMut(I, Self::Item) -> I>(self, init: I, mut f: F) -> I {
        let mut b = self.b;
        self.a.fold(init, move |init, a| {
            f(
                init,
                (
                    a,
                    match b.next() {
                        Some(b) => b,
                        None => unsafe { unreachable_unchecked() },
                    },
                ),
            )
        })
    }

    #[inline]
    fn try_fold<I, F: FnMut(I, Self::Item) -> R, R>(&mut self, init: I, mut f: F) -> R
    where
        R: Try<Output = I>,
    {
        let b = &mut self.b;
        self.a.try_fold(init, move |init: I, a: A::Item| {
            f(
                init,
                (
                    a,
                    match b.next() {
                        Some(b) => b,
                        None => unsafe { unreachable_unchecked() },
                    },
                ),
            )
        })
    }
}

// SAFETY: a and b have the same length
impl<A: DoubleEndedIterator, B: DoubleEndedIterator> DoubleEndedIterator for ZipEqEagerCheck<A, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe { both_or_none(self.a.next_back(), self.b.next_back()) }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        unsafe { both_or_none(self.a.nth_back(n), self.b.nth_back(n)) }
    }

    #[inline]
    fn try_rfold<I, F, R>(&mut self, init: I, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(I, Self::Item) -> R,
        R: Try<Output = I>,
    {
        let b = &mut self.b;
        self.a.try_rfold(init, move |init: I, a: A::Item| {
            f(
                init,
                (
                    a,
                    match b.next_back() {
                        Some(b) => b,
                        None => unsafe { unreachable_unchecked() },
                    },
                ),
            )
        })
    }

    #[inline]
    fn rfold<I, F>(self, init: I, mut f: F) -> I
    where
        Self: Sized,
        F: FnMut(I, Self::Item) -> I,
    {
        let mut b = self.b;
        self.a.rfold(init, move |init, a| {
            f(
                init,
                (
                    a,
                    match b.next_back() {
                        Some(b) => b,
                        None => unsafe { unreachable_unchecked() },
                    },
                ),
            )
        })
    }
}

impl<A: ExactSizeIterator, B: ExactSizeIterator> ExactSizeIterator for ZipEqEagerCheck<A, B> {
    fn len(&self) -> usize {
        self.a.len()
    }
}

unsafe impl<A: TrustedLen, B: Iterator> TrustedLen for ZipEqEagerCheck<A, B> {}
impl<A: FusedIterator, B: Iterator> FusedIterator for ZipEqEagerCheck<A, B> {}
