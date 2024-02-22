use eyre::Result;

pub mod scheduler;

pub trait Collect {
    fn sample(&self) -> Result<Box<dyn ToCsv>>;

    fn store(&mut self, sample: Box<dyn ToCsv>) -> Result<()>;
}

pub trait ToCsv {
    fn to_csv_row(&self) -> (u128, String);

    fn csv_headers(&self) -> String;
}