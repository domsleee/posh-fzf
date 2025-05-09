use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dirs_next::home_dir;
use posh_fzf::history::{
    get_history_all_commands, get_history_recent_commands, get_unique_reversed,
};

fn criterion_benchmark(c: &mut Criterion) {
    let history_path = home_dir()
        .expect("Could not determine home directory")
        .join(r"AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt");

    let paths = get_history_all_commands(&history_path).unwrap();

    c.bench_function("get_history_recent_commands", |b| {
        b.iter(|| get_history_recent_commands(black_box(&history_path)).unwrap())
    });
    c.bench_function("get_history_all_commands", |b| {
        b.iter(|| get_history_all_commands(black_box(&history_path)).unwrap())
    });
    c.bench_function("get_unique_reversed", |b| {
        b.iter(|| get_unique_reversed(black_box(paths.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
