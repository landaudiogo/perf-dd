use clap::ArgMatches;

pub struct Config {
    pub pid: usize,
    pub period: u64,
    pub data_directory: String,
}

impl From<ArgMatches> for Config {
    fn from(mut matches: ArgMatches) -> Self {
        let pid = matches.remove_one::<usize>("pid").expect("Required field");
        let period: u64 = matches
            .remove_one::<u64>("period")
            .expect("Missing period")
            .try_into()
            .expect("Convert usize to u64");
        let data_directory = matches
            .remove_one::<String>("data_directory")
            .expect("Required field");
        Self {
            pid,
            period,
            data_directory,
        }
    }
}