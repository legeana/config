use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};

use crate::{Unarchiver, UnarchiverBox};

type UnarchiverArc = Arc<dyn Unarchiver>;

#[derive(Default)]
pub(crate) struct Registry {
    unarchivers: HashMap<String, UnarchiverArc>,
    by_extension: HashMap<String, Vec<UnarchiverArc>>,
}

impl Registry {
    pub(crate) fn register(&mut self, unarchiver: UnarchiverBox) {
        let name = unarchiver.name();
        let unarchiver: UnarchiverArc = unarchiver.into();
        if let Some(u) = self.unarchivers.insert(name, unarchiver.clone()) {
            panic!(
                "failed to register unarchiver: {} is already registered",
                u.name()
            )
        }
        for ext in unarchiver.extensions() {
            self.by_extension
                .entry(ext)
                .and_modify(|e| e.push(unarchiver.clone()))
                .or_insert(vec![unarchiver.clone()]);
        }
    }
}

fn registry() -> &'static Registry {
    static INSTANCE: OnceLock<Registry> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut registry = Registry::default();
        register_all(&mut registry);
        registry
    })
}

fn register_all(registry: &mut Registry) {
    super::unzip::register(registry)
}

pub(crate) fn by_name(name: impl AsRef<str>) -> Result<&'static dyn Unarchiver> {
    let name = name.as_ref();
    registry()
        .unarchivers
        .get(name)
        .ok_or_else(|| anyhow!("{name:?} unarchiver does not exist"))
        .map(AsRef::as_ref)
}

pub(crate) fn by_extension(ext: impl AsRef<str>) -> Result<&'static dyn Unarchiver> {
    let ext = ext.as_ref();
    registry()
        .by_extension
        .get(ext)
        .ok_or_else(|| anyhow!("unarchiver does not exist for extension {ext:?}"))
        .map(|u| {
            u.first()
                .expect("by_extension is always non-empty")
                .as_ref()
        })
}
