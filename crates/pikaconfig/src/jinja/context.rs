use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use minijinja::State;
use minijinja::value::{Object, ObjectRepr, Value};

use super::{JError, JErrorKind, JResult};

#[derive(Debug)]
pub(crate) struct Context {
    // Filename of the template.
    pub(crate) source_file: PathBuf,
    // Directory of the template.
    pub(crate) source_dir: PathBuf,
    // Filename of the rendered file.
    pub(crate) destination_file: PathBuf,
    // Directory of the rendered file.
    pub(crate) destination_dir: PathBuf,
    // Directory of MANIFEST.
    // May be different from source_dir if render argument consists of multiple
    // path components.
    pub(crate) workdir: PathBuf,
    // MANIFEST prefix render was called in.
    // May be different from destination_dir if render_to is used.
    pub(crate) prefix: PathBuf,
}

impl Context {
    fn get_value(&self, key: &str) -> Option<Value> {
        match key {
            "source_file" => Some(Value::from_serialize(&self.source_file)),
            "source_dir" => Some(Value::from_serialize(&self.source_dir)),
            "destination_file" => Some(Value::from_serialize(&self.destination_file)),
            "destination_dir" => Some(Value::from_serialize(&self.destination_dir)),
            "workdir" => Some(Value::from_serialize(&self.workdir)),
            "prefix" => Some(Value::from_serialize(&self.prefix)),
            _ => None,
        }
    }
}

// Use from_args to implement.
// https://docs.rs/minijinja/latest/minijinja/value/fn.from_args.html
pub(crate) type BoxedMethod = Box<dyn Fn(&Context, &[Value]) -> JResult<Value> + Send + Sync>;

pub(crate) struct DynamicContext {
    ctx: Context,
    methods: HashMap<String, BoxedMethod>,
}

impl DynamicContext {
    pub(crate) fn new(ctx: Context) -> Self {
        Self {
            ctx,
            methods: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for DynamicContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicContext")
            .field("ctx", &self.ctx)
            .finish_non_exhaustive()
    }
}

impl Object for DynamicContext {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.ctx.get_value(key.as_str()?)
    }
    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        method: &str,
        args: &[Value],
    ) -> JResult<Value> {
        let Some(method) = self.methods.get(method) else {
            return Err(JError::new(
                JErrorKind::UnknownMethod,
                format!("{method} is not registered in context"),
            ));
        };
        method.as_ref()(&self.ctx, args)
    }
}
