use anyhow::Context;
use macroquad::prelude::load_string;
use std::num::NonZeroUsize;

use crate::resources::GlobalAssets;

#[derive(Clone)]
pub struct LevelInfo {
    pub number: Option<NonZeroUsize>,
    pub path: String,
    pub name: String,
}

pub async fn load_level_info() -> anyhow::Result<Vec<LevelInfo>> {
    let raw_level_info = load_string("levels.txt")
        .await
        .context("Couldn't load level index file")?;
    raw_level_info
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let mut parts = line.splitn(2, ' ');
            Ok(LevelInfo {
                number: NonZeroUsize::new(idx + 1),
                path: parts.next().context("Malformed level index")?.to_string(),
                name: parts.next().context("Malformed level index")?.to_string(),
            })
        })
        .collect()
}

impl GlobalAssets {
    pub fn get_first_level(&self) -> LevelInfo {
        self.levels[0].clone()
    }

    pub fn get_next_level(&self, info: &LevelInfo) -> LevelInfo {
        match info.number {
            None => self.get_first_level(),
            Some(n) => {
                let idx = n.get() % self.levels.len();
                self.levels[idx].clone()
            }
        }
    }

    pub fn get_level_with_path(&self, path: &str) -> LevelInfo {
        match self.levels.iter().position(|info| info.path == path) {
            Some(p) => self.levels[p].clone(),
            None => LevelInfo {
                number: None,
                path: path.to_owned(),
                name: "???".to_owned(),
            },
        }
    }
}
