use std::sync::Arc;

use lontra_strings::quote;
use minijinja::Environment;

pub(crate) fn register(env: &mut Environment, ctx: &Arc<super::Context>) {
    super::glob::register(env, ctx);
    super::header::register(env, ctx);
    env.add_filter("enquote", quote::enquote);
}
