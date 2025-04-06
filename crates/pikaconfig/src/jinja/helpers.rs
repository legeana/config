use std::sync::Arc;

use minijinja::Environment;

pub(crate) fn register(env: &mut Environment, _ctx: &Arc<super::Context>) {
    env.add_filter("enquote", quote::enquote);
}
