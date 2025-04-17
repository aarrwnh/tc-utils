#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    filename: String,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            filename: Self::LIST_FILENAME.join("."),
        }
    }
}

impl Output {
    pub const LIST_FILENAME: [&'static str; 2] = ["list", "txt"];

    fn new(filename: String) -> Self {
        Self { filename }
    }

    pub fn filename(&self) -> &str {
        "list.txt"
    }

    /// Find `list.txt` file inside input directory.
    pub fn find_list_file<P>(p: P) -> Vec<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let [filename, extension] = Self::LIST_FILENAME;
        let filename = &(filename.to_string() + ".");
        std::fs::read_dir(p)
            .expect("current dir list")
            .filter_map(|e| {
                let e = e.as_ref().unwrap();
                let path = e.path();
                if let Some(ext) = path.extension() {
                    let s = e.file_name().into_string().unwrap();
                    if ext == extension && s.contains(filename) {
                        return Some(Output::new(s));
                    } else {
                        return None;
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    }

    pub fn list_in_line(s: &str) -> bool {
        let [f, e] = Self::LIST_FILENAME;
        s.contains(f) && s.contains(&(".".to_string() + e))
    }
}

// impl From<String> for Output {
//     fn from(value: String) -> Self {
//         let path = PathBuf::from(&value).with_extension(""); // remove .txt
//         let Some(ext) = path.extension() else {
//             panic!()
//         };
//         let ver = ext.to_string_lossy().to_string();
//         let ver = ver.as_str();
//         let ver = if ver == "v1" { ver } else { "" };
//         Self::new(ver.into(), value)
//     }
// }
