use clap::Parser;
use core::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::commands::Commands;

#[derive(Debug, Clone)]
pub enum Range {
    LeftBounded(u16),
    RightBounded(u16),
    Exactly(u16),
}
impl From<u16> for Range {
    fn from(input: u16) -> Self {
        Range::Exactly(input)
    }
}

impl FromStr for Range {
    type Err = RangeParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input == ".." {
            Err(RangeParseError::FullRangeNotSupported)?
        }

        match input.matches("..").count() {
            0 | 1 => {
                let (variant, rest) = if let Some(rest) = input.strip_prefix("..") {
                    (Range::RightBounded(0), rest)
                } else if let Some(rest) = input.strip_suffix("..") {
                    (Range::LeftBounded(0), rest)
                } else {
                    (Range::Exactly(0), input)
                };

                if rest.contains("..") {
                    Err(RangeParseError::TwoWayRangeNotSupported)?
                }

                match rest.parse::<u16>() {
                    Ok(number) => match variant {
                        Range::Exactly(_) => Ok(Range::Exactly(number)),
                        Range::RightBounded(_) => Ok(Range::RightBounded(number)),
                        Range::LeftBounded(_) => Ok(Range::LeftBounded(number)),
                    },
                    Err(err) => Err(RangeParseError::ParseIntError(err)),
                }
            }
            _ => Err(RangeParseError::InvalidRange),
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Range::LeftBounded(num) => write!(f, "{num}.."),
            Range::RightBounded(num) => write!(f, "..{num}"),
            Range::Exactly(num) => write!(f, "{num}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RangeParseError {
    #[error("konachan.net does not support full ranges")]
    FullRangeNotSupported,
    #[error("konachan.net does not support two way ranges")]
    TwoWayRangeNotSupported,
    #[error("Invalid range")]
    InvalidRange,
    #[error(transparent)]
    ParseIntError(core::num::ParseIntError),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, help = "Screen width")]
    pub screen_width: Option<u16>,

    #[arg(long, help = "Screen height")]
    pub screen_height: Option<u16>,

    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    #[arg(long)]
    pub wallpapers_dir: Option<PathBuf>,
}
