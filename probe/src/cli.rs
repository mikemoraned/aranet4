use clap::Parser;
use std::time::Duration;
use duration_string::DurationString;
use std::fmt;

#[derive(Parser)]
pub struct Cli {
    #[clap(long,parse(try_from_str = parse_scan_length), default_value_t = ScanLength(Duration::from_secs(10)))]
    pub scan_length: ScanLength
}

pub struct ScanLength(pub Duration);

pub fn parse() -> Cli {
    Cli::parse()
}

impl std::fmt::Display for ScanLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = self.0;
        write!(f, "{}", DurationString::from(duration))
    }
}

fn parse_scan_length(arg: &str) -> Result<ScanLength, String> {
    let duration : Duration = DurationString::from_string(arg.into())?.into();
    Ok(ScanLength(duration))
}

