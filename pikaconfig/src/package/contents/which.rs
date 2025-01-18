use serde::Deserialize;

use crate::tera_helper;

use super::inventory;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WhichParams {
    binary: String,
}

#[derive(Clone)]
struct WhichFn;

impl inventory::RenderHelper for WhichFn {
    fn register_render_helper(&self, tera: &mut tera::Tera) {
        tera.register_function(
            "which",
            tera_helper::wrap_fn(move |args: &WhichParams| Ok(which::which(&args.binary)?)),
        );
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_render_helper(Box::new(WhichFn));
}
