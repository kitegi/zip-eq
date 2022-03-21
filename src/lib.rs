#![feature(trusted_len)]
#![feature(try_trait_v2)]
#![deny(unsafe_op_in_unsafe_fn)]

//! Zip iterator that check that its inputs have the same length.
//!
//! Two types of iterators are provided. The first one that checks that the sizes are equal
//! eagerly at the moment it's constructed. This can be checked when the iterators' lengths
//! can be known and trusted to be exact (See [`core::iter::TrustedLen`] for more details).
//! This is done using [`ZipEq::zip_eq_eager`]. Eagerly checking that the lengths are equal
//! allows the implementation to elide some bound checks, leading to faster code.  
//! Or in the case where the user knows for certain that the lengths are equal, the check can be
//! avoided with the unsafe method [`ZipEq::zip_eq_unchecked`].  
//! The second type of iterator is one that checks that the sizes are equal while it's being
//! iterated over. It can be constructed with [`ZipEq::zip_eq_lazy`].
//!
//! # Examples:
//!
//! ```
//! use zip_eq::ZipEq;
//!
//! let a = [1, 2];
//! let b = [3, 4];
//! let mut zipped = a.zip_eq_lazy(b);
//!
//! assert_eq!(zipped.next(), Some((1, 3)));
//! assert_eq!(zipped.next(), Some((2, 4)));
//! assert_eq!(zipped.next(), None);
//! ```
//!
//! ```
//! use zip_eq::ZipEq;
//!
//! let a = [1, 2];
//! let b = [3, 4];
//! let mut zipped = a.zip_eq_eager(b);
//!
//! assert_eq!(zipped.next(), Some((1, 3)));
//! assert_eq!(zipped.next(), Some((2, 4)));
//! assert_eq!(zipped.next(), None);
//! ```
//!
//! ```should_panic
//! use zip_eq::ZipEq;
//!
//! let a = [1, 2, 3];
//! let b = [3, 4];
//! let mut zipped = a.zip_eq_eager(b); // length equality check happens here.
//! ```
//!
//! ```should_panic
//! use zip_eq::ZipEq;
//!
//! let a = [1, 2, 3];
//! let b = [3, 4];
//! let mut zipped = a.zip_eq_lazy(b);
//!
//! assert_eq!(zipped.next(), Some((1, 3)));
//! assert_eq!(zipped.next(), Some((2, 4)));
//! zipped.next(); // length equality check happens here.
//! ```

use std::iter::TrustedLen;

mod eager;
mod lazy;

pub use eager::*;
pub use lazy::*;

#[cold]
fn panic_different_len() -> ! {
    panic!("ZipEq: Reached the end of one of the iterators before the other.");
}

fn size_hint_impl(a: (usize, Option<usize>), b: (usize, Option<usize>)) -> (usize, Option<usize>) {
    (
        a.0.max(b.0),
        match (a.1, b.1) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        },
    )
}

/// Trait that adds `zip_eq_*` builder functions to objects that are convertible to iterators
pub trait ZipEq {
    /// Returns a zipped iterator without checking that the lengths of the iterators are equal.
    /// The behavior is undefined if the iterators don't have the same length.
    unsafe fn zip_eq_unchecked<B>(self, b: B) -> ZipEqEagerCheck<Self::IntoIter, B::IntoIter>
    where
        Self: IntoIterator,
        B: IntoIterator;

    /// Returns a zipped iterator after checking that the lengths of the iterators are equal.
    /// # Panics
    /// Panics if `a.len() != b.len()`
    fn zip_eq_eager<B>(self, b: B) -> ZipEqEagerCheck<Self::IntoIter, B::IntoIter>
    where
        Self: IntoIterator,
        Self::IntoIter: TrustedLen,
        Self::IntoIter: ExactSizeIterator,
        B: IntoIterator,
        B::IntoIter: TrustedLen,
        B::IntoIter: ExactSizeIterator;

    /// Returns a zipped iterator without checking that the lengths of the iterators are equal.
    /// The lengths are checked during iteration to avoid undefined behavior.  
    /// In the case where the lengths are different, the behavior is unspecified and may result
    /// in panics, but will not cause undefined behavior.
    fn zip_eq_lazy<B>(self, b: B) -> ZipEqLazyCheck<Self::IntoIter, B::IntoIter>
    where
        Self: IntoIterator,
        B: IntoIterator;
}

impl<A: IntoIterator> ZipEq for A {
    unsafe fn zip_eq_unchecked<B>(self, b: B) -> ZipEqEagerCheck<A::IntoIter, B::IntoIter>
    where
        A: IntoIterator,
        B: IntoIterator,
    {
        ZipEqEagerCheck {
            a: self.into_iter(),
            b: b.into_iter(),
        }
    }

    fn zip_eq_eager<B>(self, b: B) -> ZipEqEagerCheck<A::IntoIter, B::IntoIter>
    where
        A: IntoIterator,
        A::IntoIter: TrustedLen,
        A::IntoIter: ExactSizeIterator,
        B: IntoIterator,
        B::IntoIter: TrustedLen,
        B::IntoIter: ExactSizeIterator,
    {
        let a = self.into_iter();
        let b = b.into_iter();

        if a.len() != b.len() {
            panic_different_len();
        }
        ZipEqEagerCheck { a, b }
    }

    fn zip_eq_lazy<B>(self, b: B) -> ZipEqLazyCheck<A::IntoIter, B::IntoIter>
    where
        A: IntoIterator,
        B: IntoIterator,
    {
        ZipEqLazyCheck {
            a: self.into_iter(),
            b: b.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod eager {
        use super::*;
        #[test]
        fn basic() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_eager(b);

            assert_eq!(zipped.next(), Some((1, 3)));
            assert_eq!(zipped.next(), Some((2, 4)));
            assert_eq!(zipped.next(), None);
        }

        #[test]
        #[should_panic]
        fn basic_fail() {
            let a = [1, 2, 3];
            let b = [3, 4];
            let _zipped = a.zip_eq_eager(b);
        }

        #[test]
        fn count() {
            let a = [1, 2];
            let b = [3, 4];
            let zipped = a.zip_eq_eager(b);
            assert_eq!(zipped.count(), 2);
        }

        #[test]
        fn last() {
            let a = [1, 2];
            let b = [3, 4];
            let zipped = a.zip_eq_eager(b);
            assert_eq!(zipped.last(), Some((2, 4)));
        }

        #[test]
        fn last_empty() {
            let a: [(); 0] = [];
            let b: [(); 0] = [];
            let zipped = a.zip_eq_eager(b);
            assert_eq!(zipped.last(), None);
        }

        #[test]
        fn nth() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_eager(b);
            assert_eq!(zipped.nth(1), Some((2, 4)));
        }

        #[test]
        fn nth_out_of_bounds() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_eager(b);
            assert_eq!(zipped.nth(2), None);
        }

        #[test]
        fn fold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b).fold((0, 0), |(acc_a, acc_b), (a, b)| {
                    factor += 1;
                    (acc_a + factor * a, acc_b + factor * b)
                })),
                (5, 11),
            );
        }

        #[test]
        fn try_fold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b)
                    .try_fold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                Some((5, 11)),
            );
        }

        #[test]
        fn try_fold_fail() {
            let a = [1, 2];
            let b = [3, 255];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b)
                    .try_fold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                None,
            );
        }

        #[test]
        fn rfold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b).rfold((0, 0), |(acc_a, acc_b), (a, b)| {
                    factor += 1;
                    (acc_a + factor * a, acc_b + factor * b)
                })),
                (4, 10),
            );
        }

        #[test]
        fn try_rfold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b)
                    .try_rfold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                Some((4, 10)),
            );
        }

        #[test]
        fn try_rfold_fail() {
            let a = [1, 2];
            let b = [3, 255];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_eager(b)
                    .try_rfold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                None,
            );
        }
    }

    mod lazy {
        use super::*;
        #[test]
        fn basic() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_lazy(b);

            assert_eq!(zipped.next(), Some((1, 3)));
            assert_eq!(zipped.next(), Some((2, 4)));
            assert_eq!(zipped.next(), None);
        }

        #[test]
        #[should_panic]
        fn basic_fail() {
            let a = [1, 2, 3];
            let b = [3, 4];
            let mut zipped = a.zip_eq_lazy(b);
            zipped.next();
            zipped.next();
            zipped.next();
        }

        #[test]
        fn count() {
            let a = [1, 2];
            let b = [3, 4];
            let zipped = a.zip_eq_lazy(b);
            assert_eq!(zipped.count(), 2);
        }

        #[test]
        fn last() {
            let a = [1, 2];
            let b = [3, 4];
            let zipped = a.zip_eq_lazy(b);
            assert_eq!(zipped.last(), Some((2, 4)));
        }

        #[test]
        fn last_empty() {
            let a: [(); 0] = [];
            let b: [(); 0] = [];
            let zipped = a.zip_eq_lazy(b);
            assert_eq!(zipped.last(), None);
        }

        #[test]
        fn nth() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_lazy(b);
            assert_eq!(zipped.nth(1), Some((2, 4)));
        }

        #[test]
        fn nth_out_of_bounds() {
            let a = [1, 2];
            let b = [3, 4];
            let mut zipped = a.zip_eq_lazy(b);
            assert_eq!(zipped.nth(2), None);
        }

        #[test]
        fn fold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b).fold((0, 0), |(acc_a, acc_b), (a, b)| {
                    factor += 1;
                    (acc_a + factor * a, acc_b + factor * b)
                })),
                (5, 11),
            );
        }

        #[test]
        fn try_fold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b)
                    .try_fold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                Some((5, 11)),
            );
        }

        #[test]
        fn try_fold_fail() {
            let a = [1, 2];
            let b = [3, 255];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b)
                    .try_fold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                None,
            );
        }

        #[test]
        fn rfold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b).rfold((0, 0), |(acc_a, acc_b), (a, b)| {
                    factor += 1;
                    (acc_a + factor * a, acc_b + factor * b)
                })),
                (4, 10),
            );
        }

        #[test]
        fn try_rfold() {
            let a = [1, 2];
            let b = [3, 4];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b)
                    .try_rfold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                Some((4, 10)),
            );
        }

        #[test]
        fn try_rfold_fail() {
            let a = [1, 2];
            let b = [3, 255];
            let mut factor = 0;

            assert_eq!(
                (a.zip_eq_lazy(b)
                    .try_rfold((0, 0_u8), |(acc_a, acc_b), (a, b): (u8, u8)| {
                        factor += 1;
                        Some((
                            acc_a + factor * a,
                            acc_b.checked_add(b.checked_mul(factor)?)?,
                        ))
                    })),
                None,
            );
        }
    }
}
