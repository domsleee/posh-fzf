use dirs_next::home_dir;
use posh_fzf::history::get_history_recent_commands;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let history_path = home_dir()
        .expect("Could not determine home directory")
        .join(r"AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt");
    c.bench_function("get_history_recent_commands", |b| b.iter(|| get_history_recent_commands(black_box(&history_path)).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);