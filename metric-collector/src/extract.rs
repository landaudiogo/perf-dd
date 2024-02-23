use ctrlc;
use eyre::Result;
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::configure::Config;
use crate::metrics::{
    scheduler::{Sched, SchedStat},
    Collect,
};

pub struct Extractor {
    terminate_flag: Arc<Mutex<bool>>,
    config: Config,
}

impl Extractor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            terminate_flag: Arc::new(Mutex::new(false)),
        }
    }

    fn register_sighandler(&self) {
        let terminate_flag = self.terminate_flag.clone();
        ctrlc::set_handler(move || {
            let mut terminate_flag = terminate_flag.lock().unwrap();
            *terminate_flag = true;
        })
        .expect("Error setting Ctrl-C handler");
    }

    pub fn run(self) -> Result<()> {
        self.register_sighandler();

        let proc_root_dir = format!("/proc/{:?}", self.config.pid);

        let tasks = fs::read_dir(format!("{proc_root_dir}/task"))?;
        let mut collectors: Vec<Box<dyn Collect>> = Vec::new();
        for task in tasks {
            let file_path = task?.path();
            let stem = file_path.file_stem().unwrap().to_str().unwrap();
            let tid: usize = stem.parse()?;

            collectors.push(Box::new(SchedStat::new(tid, &self.config.data_directory)));
            collectors.push(Box::new(Sched::new(tid, &self.config.data_directory)));
        }

        loop {
            if *self.terminate_flag.lock().unwrap() == true {
                break;
            }

            for collector in collectors.iter_mut() {
                let sample = collector.sample()?;
                collector.store(sample)?;
            }

            thread::sleep(Duration::from_millis(self.config.period));
        }

        Ok(())
    }
}
