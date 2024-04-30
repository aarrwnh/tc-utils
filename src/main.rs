#![allow(clippy::let_and_return)]
#![allow(clippy::single_match)]

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::{fs::File, io, io::BufRead, path::Path};

mod args;
use args::{Args, Mode};

mod list;

mod version;
use version::Version;

fn open_file(path: &str, enc: &'static Encoding) -> io::Result<Vec<String>> {
    match File::open(Path::new(path)) {
        Ok(file) => {
            let reader = DecodeReaderBytesBuilder::new()
                .encoding(Some(enc))
                .build(file);
            Ok(io::BufReader::new(reader) //
                .lines()
                .map(io::Result::unwrap)
                .collect())
        }
        Err(_) => Ok(vec![]),
    }
}

fn main() -> io::Result<()> {
    let args = Args::new();
    let cwd = std::env::current_dir()?;

    if !args.path.is_empty() && args.mode == Mode::List {
        list::handler(&args, cwd)?;
    }

    Ok(())
}

// ----------------------------------------------------------------------------------
//   - Config -
// ----------------------------------------------------------------------------------
static INFO_FOOTER_PREFIX: &str = "info:";
static LIST_FILENAME: &str = "list.txt";
