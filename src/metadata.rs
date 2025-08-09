#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Metadata {
    pub name: String,
    pub license: Vec<String>,
    pub requirements: Vec<String>,
    pub bad_license: bool,
}

impl PartialOrd for Metadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Metadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}
