use std::sync::Arc;

use anyhow::Result;
use glob_util::glob as glob_iter;
use minijinja::{Environment, Value};

use crate::jinja::{Context, map_anyhow};

fn glob(ctx: &Context, pattern: &str) -> Result<Value> {
    let paths: Vec<_> = glob_iter(&ctx.destination_dir, pattern)?.collect::<Result<_>>()?;
    log::debug!("{:?}: glob({pattern:?}) = {paths:?}", &ctx.destination_dir);
    Ok(Value::from_serialize(&paths))
}

pub(super) fn register(env: &mut Environment, ctx: &Arc<Context>) {
    let ctx = Arc::clone(ctx);
    env.add_function("glob", move |pattern: &str| {
        glob(ctx.as_ref(), pattern).map_err(map_anyhow)
    });
}
