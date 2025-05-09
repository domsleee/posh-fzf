use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    fs::OpenOptions,
    io::{self, Write},
    sync::Once,
    time::Instant,
};

thread_local! {
    pub static TIMINGS: RefCell<Timings> = RefCell::new(Timings::default());
}

#[derive(Default)]
pub struct Timings {
    data: HashMap<String, TimingNode>,
}

struct TimingNode {
    start: Instant,
    end: Option<Instant>,
}

impl Timings {
    pub fn start(&mut self, name: &str) {
        let node = TimingNode {
            start: Instant::now(),
            end: None,
        };
        self.data.insert(name.to_string(), node);
    }

    pub fn end(&mut self, name: &str) {
        if let Some(node) = self.data.get_mut(name) {
            node.end = Some(Instant::now());
        }
    }

    pub fn get_all_durations(&self) -> HashMap<String, std::time::Duration> {
        let mut durations = HashMap::new();
        for (name, node) in &self.data {
            if let Some(end) = node.end {
                durations.insert(name.clone(), end.duration_since(node.start));
            }
        }
        durations
    }
}

pub fn write_perf_logs() -> io::Result<()> {
    if !is_timings_enabled() {
        return Ok(());
    }
    let home = dirs_next::home_dir().expect("has home directory");
    let log_file_path = home.join("posh-fzf.log");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

    writeln!(file, "New log")?;
    TIMINGS
        .try_with(|t| {
            let timings = t.borrow();
            let binding = timings.get_all_durations();
            let mut durations = binding.iter().collect::<Vec<_>>();
            durations.sort_by(|a, b| b.1.cmp(a.1));
            for (name, duration) in durations {
                writeln!(file, "{duration:?}: {name}").expect("file write");
            }
        })
        .expect("timings");

    Ok(())
}

static INIT: Once = Once::new();
static mut IS_ENABLED: bool = false;
pub fn is_timings_enabled() -> bool {
    unsafe {
        INIT.call_once(|| {
            IS_ENABLED = env::var("POSH_FZF_PERF").is_ok();
        });
        IS_ENABLED
    }
}

#[macro_export]
macro_rules! timing_start {
    ($name:expr) => {
        if is_timings_enabled() {
            TIMINGS
                .try_with(|t| t.borrow_mut().start($name))
                .expect("timings")
        }
    };
}

#[macro_export]
macro_rules! timing_end {
    ($name:expr) => {
        if is_timings_enabled() {
            TIMINGS
                .try_with(|t| t.borrow_mut().end($name))
                .expect("timings")
        }
    };
}
