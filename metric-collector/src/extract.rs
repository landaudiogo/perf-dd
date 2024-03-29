use ctrlc;
use eyre::Result;
use std::{
    collections::HashMap,
    sync::mpsc::Receiver,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::configure::Config;
use crate::execute::Executor;
use crate::target::Target;

pub struct Extractor {
    terminate_flag: Arc<Mutex<bool>>,
    config: Config,
    targets: HashMap<usize, Target>,
    rx_timer: Option<Receiver<bool>>,
}

impl Extractor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            terminate_flag: Arc::new(Mutex::new(false)),
            targets: HashMap::new(),
            rx_timer: None,
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

    fn register_new_targets(&mut self, executor: &mut Executor) {
        executor
            .monitor_groups
            .iter_mut()
            .for_each(|(_, monitor_group)| {
                let new_targets = monitor_group.clone.poll_events().unwrap();
                for target in new_targets {
                    self.targets.insert(
                        target,
                        Target::new(
                            target,
                            monitor_group.futex.clone(),
                            &self.config.data_directory,
                        ),
                    );
                }
            });
    }

    fn sample_targets(&mut self) {
        let mut targets_remove = Vec::new();
        self.targets.iter_mut().for_each(|(tid, target)| {
            if let Err(_) = target.sample() {
                targets_remove.push(*tid)
            }
        });

        for tid in targets_remove {
            self.targets.remove(&tid);
        }
    }

    fn start_timer_thread(&mut self) {
        let (tx_timer, rx_timer) = std::sync::mpsc::channel::<bool>();
        self.rx_timer = Some(rx_timer);

        let period = self.config.period;
        let terminate_flag = self.terminate_flag.clone();

        thread::spawn(move || {
            while *terminate_flag.lock().unwrap() == false {
                thread::sleep(Duration::from_millis(period));
                if let Err(_) = tx_timer.send(true) {
                    break;
                };
            }
        });
    }

    pub fn run(mut self) -> Result<()> {
        self.register_sighandler();
        self.start_timer_thread();
        let mut executor = Executor::new();

        Target::search_targets_regex("jbd2", true, &self.config.data_directory, &mut executor)?
            .into_iter()
            .for_each(|target| {
                self.targets.insert(target.tid, target);
            });

        let targets = Target::search_targets_regex(
            self.config.process_name.as_ref().unwrap(),
            false,
            &self.config.data_directory,
            &mut executor,
        )?;
        targets.into_iter().for_each(|target| {
            self.targets.insert(target.tid, target);
        });

        let rx_timer = self.rx_timer.take().unwrap();
        loop {
            rx_timer.recv().unwrap();
            if *self.terminate_flag.lock().unwrap() == true {
                break;
            }

            self.sample_targets();
            self.register_new_targets(&mut executor);
        }

        Ok(())
    }
}
