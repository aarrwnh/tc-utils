extern crate chrono;
use chrono::Local;

extern crate copypasta;
use copypasta::{ClipboardContext, ClipboardProvider};

use std::collections::HashMap;
use std::path::PathBuf;
use std::{io::Result, path::Path};

use super::open_file;
use super::version::Version;
use super::Args;

pub(crate) fn handler(args: Args, cwd: PathBuf) -> Result<()> {
    let temp_data = Contents::from_temp_file(args)?.unwrap();
    let adata = match Contents::from_list_file(cwd)? {
        Some(mut data) => {
            temp_data
                .data
                .iter()
                .for_each(|a| match data.data.get_mut(a.0) {
                    Some(b) => {
                        a.1.iter().for_each(|line| {
                            let line = line.to_owned();
                            if !b.contains(&line) {
                                b.push(line);
                            }
                        });
                    }
                    None => {
                        data.data.insert(a.0.to_owned(), a.1.to_owned());
                    }
                });
            data
        }
        None => temp_data,
    };

    let adata = adata.prepare_data()?;
    let _ = std::fs::write(LIST_FILENAME, adata.join("\n"));

    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(adata[..adata.len() - 1].join("\n").trim_end().to_owned())
        .unwrap();

    Ok(())
}

const fn trim_left(c: char) -> bool {
    c == ' ' || c == '\t'
}

static METADATA_FOOTER_PREFIX: &str = "meta:";
static LIST_FILENAME: &str = "list.txt";

type DataMap = HashMap<String, Vec<String>>;

#[derive(Default, Debug, Clone, PartialEq)]
struct Contents {
    title: String,
    data: DataMap,
}

impl Contents {
    /// Parse temp file from total commander
    fn from_temp_file(args: Args) -> Result<Option<Contents>> {
        if !args.path.contains(".tmp") {
            return Ok(None);
        }
        let temp_file_contents = open_file(&args.path, encoding_rs::UTF_16LE)?;
        if temp_file_contents.is_empty() {
            return Ok(None);
        }
        let folder_name = temp_file_contents
            .first()
            .map(|path| Path::new(path).components().nth_back(1).unwrap())
            .unwrap()
            .as_os_str();

        let mut data: DataMap = HashMap::new();
        temp_file_contents.iter().for_each(|path| {
            if path.contains(LIST_FILENAME) {
                return;
            }
            let filename = Path::new(path).file_stem().unwrap().to_str().unwrap();
            let mut parts = filename.split('｜');
            let (key, subtitle) = match parts.clone().count() {
                1 => {
                    // handle if can't split
                    ("<>".to_string(), filename.to_string())
                }
                _ => {
                    let mag = parts.next().unwrap();
                    let sub = parts.last().unwrap();
                    (mag.to_string(), sub.to_string())
                }
            };

            match data.get_mut(&key) {
                Some(set) => {
                    set.push(subtitle);
                }
                None => {
                    let set = Vec::from([subtitle]);
                    data.insert(key, set);
                }
            };
        });
        Ok(Some(Self {
            title: folder_name.to_str().unwrap().to_string(),
            data,
        }))
    }

    fn from_list_file(path: PathBuf) -> Result<Option<Contents>> {
        let mut path = path.clone();
        path.push(LIST_FILENAME);
        let path = path.to_str().expect("");

        let file_contents = open_file(path, encoding_rs::UTF_8)?;
        if file_contents.is_empty() {
            return Ok(None);
        }

        let mut contents = Contents {
            title: file_contents.first().expect("title line").to_string(),
            ..Default::default()
        };

        let mut key = "";
        let end_offset = file_contents.len()
            - (if file_contents
                .last()
                .unwrap()
                .contains(METADATA_FOOTER_PREFIX)
            {
                1
            } else {
                0
            });

        for line in &file_contents[2..end_offset] {
            if line == "list.txt" || line.trim_matches(trim_left).is_empty() {
                continue;
            }

            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(list) = contents.data.get_mut(key) {
                    list.push(line.trim_matches(trim_left).to_string());
                }
            } else {
                key = &line;
                contents.data.insert(key.to_string(), Vec::new());
            }
        }

        Ok(Some(contents))
    }

    /// Format vec to readable file contents
    fn prepare_data(self) -> Result<Vec<String>> {
        let mut new_contents = vec![self.title];

        let mut keys: Vec<String> = self.data.clone().into_keys().collect();
        keys.sort_by_key(|k| k == "<>"); // move with no key to bottom

        let mut count = 0usize;
        for key in keys {
            let no_key = key.is_empty();
            new_contents.push("".into()); // add newline
            new_contents.push(key.clone());
            let mut lines = self.data.get(&key).expect("lines").clone();
            lines.sort();
            lines.iter().for_each(|line| {
                let mut line = line.to_owned().clone();
                if !no_key {
                    // add padding if filename has key
                    line.insert_str(0, "  ");
                }
                new_contents.push(line);
                count += 1;
            })
        }

        let dt = Local::now();
        let timestamp = dt.format("%Y-%m-%d_%H:%M:%S").to_string();
        new_contents.extend(vec![
            "".into(),
            "".into(),
            // versioning that might be used for parsing only
            format!(
                "  {METADATA_FOOTER_PREFIX}{}:{count}:{timestamp}",
                Version::v1
            ),
        ]);

        Ok(new_contents)
    }
}
