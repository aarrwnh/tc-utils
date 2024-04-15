#![allow(clippy::let_and_return)]
#![allow(clippy::single_match)]

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::io::BufRead;
use std::{fs::File, io::BufReader, io::Result, path::Path};

mod args;
use args::{Args, Mode};

mod list;
mod version;

fn open_file(path: &str, enc: &'static Encoding) -> Result<Vec<String>> {
    match File::open(Path::new(path)) {
        Ok(file) => {
            let reader = DecodeReaderBytesBuilder::new()
                .encoding(Some(enc))
                .build(file);
            Ok(BufReader::new(reader) //
                .lines()
                .map(Result::unwrap)
                .collect())
        }
        Err(_) => Ok(vec![]),
    }
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1);
    let args = Args::parse(args);
    let cwd = std::env::current_dir()?;

    if !args.path.is_empty() && args.mode == Mode::List {
        list::handler(args, cwd)?;
    }

    Ok(())
}
