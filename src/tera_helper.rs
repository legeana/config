use std::{collections::HashMap, marker::PhantomData};

use anyhow::Result;
use serde::de::value::MapDeserializer;
use serde::Deserialize;
use tera::{Function, Value};

pub trait SimpleFunction<Params>
where
    Params: Deserialize<'static> + Send + Sync,
{
    fn call(&self, args: &Params) -> Result<Value>;
}

pub struct WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: Deserialize<'static> + Send + Sync,
{
    wrapped: T,
    phantom: PhantomData<Params>,
}

impl<T, Params> Function for WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: Deserialize<'static> + Send + Sync,
{
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        // https://github.com/serde-rs/serde/issues/1739
        let args = Params::deserialize(MapDeserializer::new(args.clone().into_iter()))?;
        match self.wrapped.call(&args) {
            Ok(result) => Ok(result),
            Err(err) => Err(tera::Error::msg(err.to_string())),
        }
    }
}

pub fn wrap_function<T, Params>(func: T) -> WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: Deserialize<'static> + Send + Sync,
{
    WrappedFunction {
        wrapped: func,
        phantom: PhantomData,
    }
}
