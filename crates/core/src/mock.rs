// SPDX-License-Identifier: GPL-3.0-or-later
//! Provider mock: lê `fixtures/mock/*.json` (ordem alfabética). Cada arquivo é um `Snapshot`
//! ou um array de `Snapshot`. A cada chamada de `next` a apuração "avança" um quadro
//! (e permanece no último).

use std::path::Path;

use crate::error::{Error, Result};
use crate::model::{Fonte, Snapshot};
use crate::parser::now_iso;

#[derive(Debug, Clone)]
pub struct MockProvider {
    frames: Vec<Snapshot>,
    next: usize,
}

impl MockProvider {
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let mut paths: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        paths.sort();
        let mut frames = Vec::new();
        for p in paths {
            let text = std::fs::read_to_string(&p)?;
            let value: serde_json::Value = serde_json::from_str(&text)?;
            if value.is_array() {
                frames.extend(serde_json::from_value::<Vec<Snapshot>>(value)?);
            } else {
                frames.push(serde_json::from_value::<Snapshot>(value)?);
            }
        }
        Self::from_frames(frames)
    }

    pub fn from_frames(frames: Vec<Snapshot>) -> Result<Self> {
        if frames.is_empty() {
            return Err(Error::Config("fixtures/mock sem snapshots".into()));
        }
        Ok(Self { frames, next: 0 })
    }

    pub fn avancar(&mut self) -> Snapshot {
        let mut s = self.frames[self.next].clone();
        if self.next + 1 < self.frames.len() {
            self.next += 1;
        }
        s.fonte = Fonte::Mock;
        s.gerado_em = now_iso();
        s.desatualizado = false;
        s
    }
}
