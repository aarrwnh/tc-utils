use chrono::Local;
use regex::Regex;
use clipboard_win::{formats, set_clipboard};

use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};
use std::path::PathBuf;
use std::{
    io::{Error, ErrorKind},
    path::Path,
};

use crate::version::Version;
use crate::Args;
use crate::*;

pub(crate) fn handler(args: Args, cwd: PathBuf) -> Result<(), Error> {
    let temp_data = Contents::from_temp_file(args.path)?.unwrap();
    let adata = match Contents::from_list_file(cwd)? {
        Some(mut data) => {
            temp_data
                .data
                .iter()
                .for_each(|(key, lines)| match data.data.get_mut(key) {
                    Some(b) => {
                        lines.iter().for_each(|line| {
                            let line = line.to_owned();
                            if !b.contains(&line) {
                                b.push(line);
                            }
                        });
                    }
                    None => {
                        data.data.insert(key.to_owned(), lines.to_owned());
                    }
                });
            data
        }
        None => temp_data,
    };

    if let Some(adata) = match adata.prepare_data() {
        Ok((data, Some(err))) => {
            eprintln!("{err}");
            Some(data)
        }
        Ok((data, None)) => {
            std::fs::write(LIST_FILENAME, data.join("\n")).unwrap();
            Some(data)
        }
        Err(err) => {
            eprintln!("{err}");
            None
        }
    } {
        let text = adata[..adata.len() - 1].join("\n").trim_end().to_owned();
        println!("{text}");

        if !args.ignore_clipboard {
            set_clipboard(formats::Unicode, text).expect("To set clipboard");
        }
    }

    Ok(())
}

const fn trim_left(c: char) -> bool {
    c == ' ' || c == '\t'
}

type DataMap = HashMap<String, Vec<String>>;

#[derive(Default, Debug, Clone, PartialEq)]
struct Contents {
    title: String,
    data: DataMap,
    count: usize,
}

impl Contents {
    /// Parse temp file from total commander
    fn from_temp_file(path: String) -> Result<Option<Self>, Error> {
        if !path.contains(".tmp") {
            return Err(ErrorKind::NotFound.into());
        }
        let temp_file_contents = open_file(&path, encoding_rs::UTF_16LE)?;
        if temp_file_contents.is_empty() {
            return Err(ErrorKind::UnexpectedEof.into());
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
                    data.insert(key, Vec::from([subtitle]));
                }
            };
        });
        Ok(Some(Self {
            title: folder_name.to_str().unwrap().into(),
            data,
            count: 0,
        }))
    }

    fn from_list_file(path: PathBuf) -> Result<Option<Self>, Error> {
        let mut path = path.clone();
        path.push(LIST_FILENAME);
        let path = path.to_str().expect("");

        let lines = open_file(path, encoding_rs::UTF_8)?;
        if lines.is_empty() {
            return Ok(None);
        }

        let mut contents = Self {
            title: lines.first().expect("title line").to_string(),
            ..Default::default()
        };

        let footer = lines.last().unwrap();
        let mut end_offset = lines.len();
        if footer.contains(METADATA_FOOTER_PREFIX) {
            end_offset.sub_assign(1);
            contents.count = footer
                .split(':')
                .collect::<Vec<_>>()
                .get(2)
                .expect("count found at position 2")
                .to_string()
                .parse::<usize>()
                .unwrap_or_default();
        }

        let mut key = "";
        let mut i = 1;
        while i < end_offset {
            let mut line = &lines[i];
            i.add_assign(1);

            if *line == contents.title // in old files
                || line == LIST_FILENAME
                || line.trim_matches(trim_left).is_empty()
            {
                continue;
            }

            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(list) = contents.data.get_mut(key) {
                    loop {
                        let trimmed_line = line.trim_matches(trim_left);
                        if trimmed_line.is_empty() {
                            break;
                        }
                        if !list.contains(&trimmed_line.to_string()) {
                            list.push(trimmed_line.into());
                        }
                        line = &lines[i];
                        i.add_assign(1);
                    }
                }
            } else {
                key = &line;
                // println!("{i} : {key}");
                contents.data.entry(key.into()).or_default();
            }
        }

        let mut stray_lines = Vec::new();
        for key in contents.data.to_owned().keys() {
            if let Some(val) = contents.data.get(key) {
                if val.is_empty() {
                    stray_lines.push(key.to_string());
                    contents.data.remove_entry(key);
                }
            }
        }

        if !stray_lines.is_empty() {
            contents
                .data
                .entry("<>".into())
                .or_default()
                .extend(stray_lines);
        }

        Ok(Some(contents))
    }

    /// Format vec to readable file contents
    fn prepare_data(self) -> Result<(Vec<String>, Option<Error>), Error> {
        let re_date = Regex::new(r"\((\d{4}).(\d{1,2}).(\d{1,2}).?\)").unwrap();
        let re_chapter_no = Regex::new(r"第?(\d+)話").unwrap();

        let mut new_contents = vec![self.title];

        let mut keys: Vec<String> = self.data.clone().into_keys().collect();
        keys.sort_by_key(|k| k == "<>"); // move with no key to bottom

        let mut count = 0usize;
        for key in keys {
            let has_key = !key.is_empty();
            new_contents.push("".into()); // add newline
            new_contents.push(key.clone());
            let mut lines = self.data.get(&key).expect("lines").clone();

            // try very naive sorting
            lines.sort_by_key(|line| {
                if let Some(chapter) = re_chapter_no.captures(line) {
                    return chapter[1].parse::<i32>().unwrap().abs();
                }
                if let Some(date) = re_date.captures(line) {
                    let d = format!("{}{:0>2}{:0>2}", &date[1], &date[2], &date[3]);
                    return d.parse::<i32>().unwrap().abs();
                }
                0
            });

            lines.iter().for_each(|line| {
                let mut line = line.to_owned().clone();
                if has_key {
                    // add padding
                    line.insert_str(0, "  ");
                }
                new_contents.push(line);
                count += 1;
            })
        }

        if count == self.count {
            return Ok((
                new_contents,
                Some(Error::new(ErrorKind::Other, "no changes to be made")),
            ));
        }

        let dt = Local::now();
        let timestamp = dt.format("%Y-%m-%d_%H:%M:%S").to_string();
        new_contents.extend(vec![
            "".into(),
            "".into(),
            format!(
                "/*  {METADATA_FOOTER_PREFIX}{}:{count}:{timestamp}  */",
                // versioning that in the future might be used for parsing only?
                Version::v1
            ),
        ]);

        Ok((new_contents, None))
    }
}
