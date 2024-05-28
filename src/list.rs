use chrono::Local;
use clipboard_win::{formats, set_clipboard};
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;

use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};

use crate::{args::SortStrategy, *};
use crate::{Error, Result};

static RE_DATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\((\d{4}).(\d{1,2}).(\d{1,2}).?\)?").unwrap());
static RE_CHAPTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"第?(\d+)話").unwrap());

pub(crate) fn handler(args: &Args, cwd: PathBuf) -> Result<()> {
    let temp_data = Contents::from_temp_file(&args.path)?.unwrap();
    let adata = match Contents::from_list_file(cwd, args)? {
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

    if let Some(adata) = match adata.prepare_data(args) {
        Ok((data, Some(err), _)) => {
            eprintln!("{err}");
            Some(data)
        }
        Ok((data, None, output)) => {
            if !args.dry_run {
                fs::write(output.to_filename(), data.join("\n")).unwrap();
            }
            Some(data)
        }
        Err(err) => {
            eprintln!("{err}");
            None
        }
    } {
        let text = adata[..adata.len() - 1].join("\n").trim_end().to_owned();
        if !args.ignore_clipboard {
            println!("{text}");
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
    label: String,
    data: DataMap,
    count: usize,
}

impl Contents {
    /// Parse temp file from total commander
    fn from_temp_file(path: &str) -> Result<Option<Self>> {
        if !path.contains(".tmp") {
            return Err(Error::NotFound);
        }
        let temp_file_contents = open_file(path, encoding_rs::UTF_16LE)?;
        if temp_file_contents.is_empty() {
            return Err(Error::UnexpectedEof);
        }
        let folder_name = temp_file_contents
            .first()
            .map(|path| Path::new(path).components().nth_back(1).unwrap())
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        let mut data: DataMap = HashMap::new();
        temp_file_contents.iter().for_each(|path| {
            if list_in_line(path) {
                return;
            }
            let p = Path::new(path);
            let filename = if p.is_dir() {
                p.file_name()
            } else {
                p.file_stem()
            }
            .unwrap()
            .to_str()
            .unwrap();

            let mut parts = filename.split('｜');
            let (key, subtitle) = match parts.clone().count() {
                1 => {
                    // handle if can't split
                    ("<>".to_string(), filename.to_string())
                }
                _ => {
                    let mag = parts.next().unwrap();
                    let sub = parts.last().unwrap();
                    let sub = sub.trim_start_matches(&folder_name).to_string();
                    (mag.to_string(), sub)
                }
            };

            let subtitle = subtitle.trim_matches(trim_left).to_owned();

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
            label: folder_name,
            data,
            count: 0,
        }))
    }

    fn from_list_file(path: PathBuf, args: &Args) -> Result<Option<Self>> {
        let path = path.join(Output::LIST_FILENAME.join("."));
        let lines = open_file(path.to_str().unwrap(), encoding_rs::UTF_8)?;

        if lines.is_empty() {
            return Ok(None);
        }

        if !args.dry_run {
            backup_file(path)?;
        }

        let mut contents = Self {
            label: lines.first().expect("line containing label").to_string(),
            ..Default::default()
        };

        let footer = lines.last().unwrap();
        let mut end_offset = lines.len();
        if footer.contains(INFO_FOOTER_PREFIX) || footer.starts_with("/*") {
            end_offset -= 1;
            contents.count = footer
                .split(if footer.contains(',') { ',' } else { ':' })
                .collect::<Vec<_>>()
                .get(2)
                .expect("count found at position 2")
                .to_string()
                .parse::<usize>()
                .unwrap_or(0);
        }

        let mut key = "";
        let mut i = 1;

        while i < end_offset {
            let mut line = &lines[i];
            i += 1;

            if *line == contents.label // in old files
                || list_in_line(line)
                || line.trim_matches(trim_left).is_empty()
            {
                continue;
            }

            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(list) = contents.data.get_mut(key) {
                    while i < lines.len() {
                        let trimmed_line = line.trim_matches(trim_left);
                        if trimmed_line.is_empty() {
                            break;
                        }
                        if !list.contains(&trimmed_line.to_string()) {
                            list.push(trimmed_line.into());
                        }
                        line = &lines[i];
                        i += 1;
                    }
                }
            } else {
                key = line;
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
    fn prepare_data(self, args: &Args) -> Result<(Vec<String>, Option<Error>, Output)> {
        let mut new_contents = vec![self.label];

        let mut keys: Vec<String> = self.data.clone().into_keys().collect();
        // keys.sort_by_key(|k| k == "<>"); // move with no key to bottom
        keys.sort();

        let one_key = keys.len() == 1 && keys[0] == "<>";
        let mut count = 0usize;
        for key in keys {
            let has_key = !key.is_empty();
            if !one_key {
                new_contents.push("".into()); // add newline
                new_contents.push(key.clone());
            }
            let mut lines = self.data.get(&key).expect("lines").clone();

            sort(&mut lines, &args.sort);

            lines.iter().for_each(|line| {
                let mut line = line.to_owned().clone();
                if has_key {
                    // add padding
                    line.insert_str(0, "  ");
                }
                if !new_contents.contains(&line) {
                    new_contents.push(line);
                    count += 1;
                }
            })
        }

        let filelist = Output::find_list_file();
        let output = match filelist.len() {
            1 => filelist.last().unwrap().clone(),
            _ => Output::default(),
        };

        if count == self.count {
            return Ok((new_contents, Some(Error::NoChange), output));
        }

        new_contents.push(format!(
            "\n\n/*  {INFO_FOOTER_PREFIX}{version},{count},{timestamp}  */",
            version = output.get_version(),
            timestamp = Local::now().format("%Y-%m-%d_%H:%M:%S")
        ));

        Ok((new_contents, None, output))
    }
}

/// try very naive sorting
fn sort(lines: &mut [String], strategy: &SortStrategy) {
    use SortStrategy::*;
    if *strategy == Name {
        lines.sort();
    } else {
        let mut sorted = false;
        lines.sort_unstable_by_key(|line| {
            let a = line.to_ascii_lowercase();
            match strategy {
                Name => {}
                None => {
                    if let Some(chapter) = RE_CHAPTER.captures(line) {
                        sorted = true;
                        return (chapter[1].parse::<i32>().unwrap().abs(), a);
                    }
                }
                Date => {
                    if let Some(date) = RE_DATE.captures(line) {
                        let d = format!("{}{:0>2}{:0>2}", &date[1], &date[2], &date[3]);
                        sorted = true;
                        return (d.parse::<i32>().unwrap().abs(), a);
                    }
                }
            }
            (0, a)
        });

        if !sorted {
            lines.sort();
        }
    }
}

fn random_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}

fn list_in_line(s: &str) -> bool {
    let [f, e] = Output::LIST_FILENAME;
    s.contains(f) && s.contains(&(".".to_string() + e))
}

fn backup_file(path: PathBuf) -> io::Result<()> {
    // TODO?: this is fine as long ramdisk is used
    let temp_path = env::temp_dir().join("_tc");
    fs::create_dir_all(&temp_path)?;
    fs::copy(path, temp_path.join(random_string(8) + "-list.txt.bak"))?;
    Ok(())
}
