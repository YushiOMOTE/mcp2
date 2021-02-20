use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone, Debug)]
pub struct IgnoreDetail {
    name: String,
    image_only: bool,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum IgnoreEntry {
    IgnoreUser(String),
    Ignore(IgnoreDetail),
}

impl<'a> From<&'a IgnoreEntry> for IgnoreDetail {
    fn from(e: &'a IgnoreEntry) -> Self {
        match e {
            IgnoreEntry::IgnoreUser(u) => IgnoreDetail {
                name: u.into(),
                image_only: false,
            },
            IgnoreEntry::Ignore(i) => i.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct Ignore(Vec<IgnoreEntry>);

impl Ignore {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();

        serde_yaml::from_reader(
            std::fs::File::open(path)
                .with_context(|| format!("cannot open ignore list at {}", path.display()))?,
        )
        .context("cannot parse ignore list")
    }

    pub fn skip(&self, user: &str) -> bool {
        match self.get(user) {
            Some(e) => !e.image_only,
            None => false,
        }
    }

    pub fn skip_image(&self, user: &str) -> bool {
        self.get(user).is_some()
    }

    fn get(&self, user: &str) -> Option<IgnoreDetail> {
        self.0
            .iter()
            .find(|e| match e {
                IgnoreEntry::IgnoreUser(u) if u == user => true,
                IgnoreEntry::Ignore(i) if i.name == user => true,
                _ => false,
            })
            .map(|e| e.into())
    }
}
