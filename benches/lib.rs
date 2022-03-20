use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zip_strict::Zip;

fn add_slices_std(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip(a)
        .zip(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

fn add_slices_eager(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip_eq_eager(a)
        .zip_eq_eager(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

fn add_slices_lazy(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.iter_mut()
        .zip_eq_eager(a)
        .zip_eq_eager(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

fn add_slices_std_chunked(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.chunks_mut(64)
        .flatten()
        .zip(a)
        .zip(b)
        .for_each(|((o, a), b)| *o = *a + *b);
}

fn add_slices_zip_chunked(out: &mut [f64], a: &[f64], b: &[f64]) {
    unsafe {
        out.chunks_mut(64)
            .flatten()
            .zip_eq_unchecked(a)
            .zip_eq_unchecked(b)
    }
    .for_each(|((o, a), b)| *o = *a + *b);
}

fn add_slices_zip_lazy_chunked(out: &mut [f64], a: &[f64], b: &[f64]) {
    out.chunks_mut(64)
        .flatten()
        .zip_eq_lazy(a)
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

    c.bench_function("add chunks std", |b| {
        b.iter(|| add_slices_std_chunked(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("add chunks zip", |b| {
        b.iter(|| add_slices_zip_chunked(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
    c.bench_function("add chunks zip lazy", |b| {
        b.iter(|| add_slices_zip_lazy_chunked(black_box(&mut out), black_box(&lhs), black_box(&rhs)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
