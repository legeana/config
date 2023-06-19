use anyhow::Result;
use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use serde_json::Value as Json;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
struct AsStdError(#[from] anyhow::Error);

pub trait SimpleHelper {
    fn call_inner(&self, params: &[&Json]) -> Result<Json>;
}

// Foreign trait can only implement a local type.
struct Wrapper<T>(T)
where
    T: SimpleHelper;

impl<T> HelperDef for Wrapper<T>
where
    T: SimpleHelper,
{
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let params: Vec<&Json> = h.params().iter().map(|j| j.value()).collect();
        let result = self.0.call_inner(&params).map_err(|err| {
            RenderError::from_error("", AsStdError(err))
        })?;
        Ok(ScopedJson::Derived(result))
    }
}

pub fn wrap<T>(t: T) -> Box<dyn HelperDef + Send + Sync>
where
    T: SimpleHelper + Send + Sync + 'static,
{
    Box::new(Wrapper(t))
}
