use std::cell;
use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;
use sonic_csv::csv::CsvCursor;
use sonic_csv::csv_simd::CsvSimdCursor;
use sonic_csv::csv_simd_bespoke::CsvSimdBespokeCursor;

fn gen_csv_data() -> String {
    //let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut output = String::new();
    let height = 2000;
    let width = 1000;
    for i in 0..height{
        let mut first = false;
        for j in 0..width{
            let val = i*height + j;
            if !first {
                write!(&mut output, ",").expect("");
            }
            write!(&mut output, "{val}").expect("");
            first = false;
        }
        write!(&mut output, "\r\n").expect("");
    }
    
    output
}


fn gen_csv_data_wide() -> String {
    //let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut output = String::new();
    let height = 2000;
    let width = 1000;
    for i in 0..height{
        let mut first = false;
        for j in 0..width{
            let val = i*height + j;
            if !first {
                write!(&mut output, ",").expect("");
            }
            write!(&mut output, "aaaaaaaaaaaaaaaaaaaa").expect("");
            first = false;
        }
        write!(&mut output, "\r\n").expect("");
    }
    
    output

}



fn gen_csv_data_wide_wide() -> String {
    //let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut output = String::new();
    let height = 2000;
    let width = 1000;
    for i in 0..height{
        let mut first = false;
        for j in 0..width{
            let val = i*height + j;
            if !first {
                write!(&mut output, ",").expect("");
            }
            write!(&mut output, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").expect("");
            first = false;
        }
        write!(&mut output, "\r\n").expect("");
    }
    
    output
}

fn read_all_csv(csv: &str)->u64{
    let mut c = CsvCursor::new(csv.as_bytes());
    let mut finished = false;
    let mut cell_count = 0;
    while !finished {
        while let Some(_) = c.next_value(){
            cell_count += 1;
        }
        finished = !c.advance_line();
    }
    cell_count
}

fn read_all_csv_simd(csv: &str)->u64{
    let mut c = CsvSimdCursor::new(csv.as_bytes());
    let mut finished = false;
    let mut cell_count = 0;
    while !finished {
        while let Some(_) = c.next_value(){
            cell_count += 1;
        }
        finished = !c.advance_line();
    }
    cell_count
}

fn read_all_csv_simd_bespoke(csv: &str)->u64{
    let mut c = CsvSimdBespokeCursor::new(csv.as_bytes());
    let mut finished = false;
    let mut cell_count = 0;
    while !finished {
        while let Some(_) = c.next_value(){
            cell_count += 1;
        }
        finished = !c.advance_line();
    }
    cell_count
}

fn simple_data_benchmark(c: &mut Criterion) {
    let d = gen_csv_data();
    c.bench_function("[numbers] base", |b| b.iter(|| read_all_csv(black_box(d.as_str()))));
    c.bench_function("[numbers] memchr", |b| b.iter(|| read_all_csv_simd(black_box(d.as_str()))));
    c.bench_function("[numbers] bespoke", |b| b.iter(|| read_all_csv_simd_bespoke(black_box(d.as_str()))));
    
    let e = gen_csv_data_wide();
    c.bench_function("[wide] base", |b| b.iter(|| read_all_csv(black_box(e.as_str()))));
    c.bench_function("[wide] memchr", |b| b.iter(|| read_all_csv_simd(black_box(e.as_str()))));
    c.bench_function("[wide] bespoke", |b| b.iter(|| read_all_csv_simd_bespoke(black_box(e.as_str()))));
    
    let f = gen_csv_data_wide_wide();
    c.bench_function("[wide wide] base", |b| b.iter(|| read_all_csv(black_box(f.as_str()))));
    c.bench_function("[wide wide] memchr", |b| b.iter(|| read_all_csv_simd(black_box(f.as_str()))));
    c.bench_function("[wide wide] bespoke", |b| b.iter(|| read_all_csv_simd_bespoke(black_box(f.as_str()))));
}


criterion_group!(benches, simple_data_benchmark);
criterion_main!(benches);