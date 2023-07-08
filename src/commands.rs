use clap::{arg, command, Subcommand};

use crate::{
    cli::Range,
    rating::{Category, KonachanRatingFilter, Rating},
};
use core::str::FromStr;

pub mod download;
pub mod review;
pub mod set;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(arg_required_else_help = false)]
    Download {
        #[arg(long, value_parser = Range::from_str)]
        download_width: Option<Range>,

        #[arg(long, value_parser = Range::from_str)]
        download_height: Option<Range>,

        #[arg(long)]
        tags: Option<String>,

        #[arg(long, value_parser = KonachanRatingFilter::from_str, default_value_t = KonachanRatingFilter::Safe)]
        rating: KonachanRatingFilter,
    },
    #[command(arg_required_else_help = false)]
    Set {
        #[command(subcommand)]
        subcommand: SetSubcommand,
    },
    Review {
        #[command(subcommand)]
        subcommand: ReviewSubcommand,
    },
}
#[derive(Subcommand, Debug)]
pub enum ReviewSubcommand {
    Current,
    Liked,
    Disliked,
    Borked,
}

#[derive(Subcommand, Debug)]
pub enum HistorySubcommand {
    Previous,
    Current,
    Next,
}

#[derive(Subcommand, Debug)]
pub enum SetSubcommand {
    #[command(flatten)]
    History(HistorySubcommand),
    Random {
        #[arg(long, value_parser = Rating::from_str, default_value_t = Rating::Safe)]
        rating: Rating,
        #[arg(long, value_parser = Category::from_str, default_value_t = Category::Liked)]
        category: Category,
    },
    File {
        path: String,
    },
    Md5 {
        md5: String,
    },
}
