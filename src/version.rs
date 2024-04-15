#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Version {
    v1,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
