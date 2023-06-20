use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context, Result};
use serde::de::{value::MapDeserializer, DeserializeOwned};
use tera::{Filter, Function, Value};

fn deserialize_args<T>(args: &HashMap<String, Value>) -> Result<T>
where
    T: DeserializeOwned + Send + Sync,
{
    // https://github.com/serde-rs/serde/issues/1739
    T::deserialize(MapDeserializer::new(args.clone().into_iter()))
        .context("failed to deserialize args")
}

pub trait SimpleFunction<Params>
where
    Params: DeserializeOwned + Send + Sync,
{
    fn call(&self, args: &Params) -> Result<Value>;
}

pub struct WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
{
    wrapped: T,
    phantom: PhantomData<Params>,
}

impl<T, Params> Function for WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
{
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let wrap_result = || {
            let args: Params = deserialize_args(args)?;
            self.wrapped.call(&args)
        };
        match wrap_result() {
            Ok(result) => Ok(result),
            Err(err) => Err(tera::Error::msg(err.to_string())),
        }
    }
}

pub fn wrap_function<T, Params>(func: T) -> WrappedFunction<T, Params>
where
    T: SimpleFunction<Params> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
{
    WrappedFunction {
        wrapped: func,
        phantom: PhantomData,
    }
}

pub trait SimpleFilter<V, Params>
where
    Params: DeserializeOwned + Send + Sync,
    V: DeserializeOwned + Send + Sync,
{
    fn call(&self, value: &V, args: &Params) -> Result<Value>;
}

pub struct WrappedFilter<T, V, Params>
where
    T: SimpleFilter<V, Params> + Send + Sync,
    V: DeserializeOwned + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
{
    wrapped: T,
    value: PhantomData<V>,
    params: PhantomData<Params>,
}

impl<T, V, Params> Filter for WrappedFilter<T, V, Params>
where
    T: SimpleFilter<V, Params> + Send + Sync,
    V: DeserializeOwned + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
{
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let wrap_result = || {
            let value = serde_json::from_value(value.to_owned())?;
            let args: Params = deserialize_args(args)?;
            self.wrapped.call(&value, &args)
        };
        match wrap_result() {
            Ok(result) => Ok(result),
            Err(err) => Err(tera::Error::msg(err.to_string())),
        }
    }
}
