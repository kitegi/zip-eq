use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::{
    vec_deque::{Iter, IterMut},
    VecDeque,
};
use zip_eq::Zip;

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

fn criterion_benchmark(c: &mut Criterion) {
    let n = 0x1000;

    let mut out = vec![0.0; n];
    let lhs = vec![0.0; n];
    let rhs = vec![0.0; n];

    c.bench_function("add slices std", |b| {
        b.iter(|| add_slices_std(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("add slices eager", |b| {
        b.iter(|| add_slices_eager(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("add slices lazy", |b| {
        b.iter(|| add_slices_lazy(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });

    let mut out: VecDeque<_> = vec![0.0; n].into();
    let lhs: VecDeque<_> = vec![0.0; n].into();
    let rhs: VecDeque<_> = vec![0.0; n].into();

    c.bench_function("add chunks std", |b| {
        b.iter(|| {
            add_slices_std_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });
    c.bench_function("add chunks eager", |b| {
        b.iter(|| {
            add_slices_zip_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });
    c.bench_function("add chunks lazy", |b| {
        b.iter(|| {
            add_slices_zip_lazy_chunked(
                black_box(out.iter_mut()),
                black_box(lhs.iter()),
                black_box(rhs.iter()),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
