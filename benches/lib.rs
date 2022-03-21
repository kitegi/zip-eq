#![deny(unsafe_op_in_unsafe_fn)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lipsum::lipsum;
use std::{
    collections::{
        vec_deque::{Iter, IterMut},
        VecDeque,
    },
    str::Chars,
};
use zip_eq::ZipEq;

#[inline(never)]
fn add_slices_std(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip(a)
        .zip(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_slices_eager(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip_eq_eager(a)
        .zip_eq_eager(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_slices_lazy(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip_eq_eager(a)
        .zip_eq_eager(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_slices_std_chunked(out: IterMut<'_, f64>, a: Iter<'_, f64>, b: Iter<'_, f64>) {
    out.zip(a).zip(b).for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_slices_zip_chunked(out: IterMut<'_, f64>, a: Iter<'_, f64>, b: Iter<'_, f64>) {
    out.zip_eq_eager(a)
        .zip_eq_eager(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_slices_zip_lazy_chunked(out: IterMut<'_, f64>, a: Iter<'_, f64>, b: Iter<'_, f64>) {
    out.zip_eq_lazy(a)
        .zip_eq_lazy(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

#[inline(never)]
fn add_chars_std(a: Chars<'_>, b: Chars<'_>) -> u32 {
    a.zip(b).fold(0, |acc, (a, b)| acc + a as u32 + b as u32)
}

#[inline(never)]
fn add_chars_lazy(a: Chars<'_>, b: Chars<'_>) -> u32 {
    a.zip_eq_lazy(b)
        .fold(0, |acc, (a, b)| acc + a as u32 + b as u32)
}

#[inline(never)]
unsafe fn add_chars_eager(a: Chars<'_>, b: Chars<'_>) -> u32 {
    unsafe { a.zip_eq_unchecked(b) }.fold(0, |acc, (a, b)| acc + a as u32 + b as u32)
}

fn criterion_benchmark(c: &mut Criterion) {
    let n = 0x1000;

    let mut out = vec![0.0; n];
    let lhs = vec![0.0; n];
    let rhs = vec![0.0; n];

    c.bench_function("slices std", |b| {
        b.iter(|| add_slices_std(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("slices eager", |b| {
        b.iter(|| add_slices_eager(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("slices lazy", |b| {
        b.iter(|| add_slices_lazy(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });

    let mut out: VecDeque<_> = vec![0.0; n].into();
    let lhs: VecDeque<_> = vec![0.0; n].into();
    let rhs: VecDeque<_> = vec![0.0; n].into();

    c.bench_function("chunks std", |b| {
        b.iter(|| {
            add_slices_std_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });
    c.bench_function("chunks eager", |b| {
        b.iter(|| {
            add_slices_zip_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });
    c.bench_function("chunks lazy", |b| {
        b.iter(|| {
            add_slices_zip_lazy_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });

    let s = lipsum(n);
    c.bench_function("unknown len std", |b| {
        b.iter(|| black_box(add_chars_std(black_box(s.chars()), black_box(s.chars()))))
    });
    c.bench_function("unknown len eager", |b| {
        b.iter(|| black_box(unsafe { add_chars_eager(black_box(s.chars()), black_box(s.chars())) }))
    });
    c.bench_function("unknown len lazy", |b| {
        b.iter(|| black_box(add_chars_lazy(black_box(s.chars()), black_box(s.chars()))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
