use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// file version:
    /// list.txt -- old v0
    /// list.v1.txt -- new format
    version: String,
    filename: String,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            version: "v1".into(),
            filename: Self::LIST_FILENAME.join("."),
        }
    }
}

impl Output {
    pub const LIST_FILENAME: [&'static str; 2] = ["list", "txt"];

    fn new(version: String, filename: String) -> Self {
        Self { version, filename }
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    fn set_version(&mut self, s: String) {
        self.version = s;
    }

    pub fn to_filename(&self) -> String {
        let ver = self.get_version();
        format!("list{}{}.txt", if ver.is_empty() { "" } else { "." }, ver)
    }

    pub fn find_list_file() -> Vec<Self> {
        std::fs::read_dir(".")
            .expect("current dir list")
            .filter_map(|e| {
                let e = e.as_ref().unwrap();
                let path = e.path();
                if let Some(ext) = path.extension() {
                    let s = e.file_name().into_string().unwrap();
                    if ext == "txt" && s.contains("list") {
                        return Some(Output::from(s));
                    } else {
                        return None;
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    }
}

impl From<String> for Output {
    fn from(value: String) -> Self {
        let mut v = Self::new("".into(), value.clone());
        match PathBuf::from(value.to_owned())
            .with_extension("") // remove .txt
            .extension()
        {
            Some(ext) => match ext.to_string_lossy().to_string().as_str() {
                ver @ "v1" => {
                    v.set_version(ver.into());
                }
                _ => {}
            },
            _ => {}
        };
        v
    }
}
