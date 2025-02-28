#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Metadata {
    pub name: String,
    pub license: Vec<String>,
    pub requirements: Vec<String>,
    pub bad_license: bool,
}

impl Ord for Metadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
