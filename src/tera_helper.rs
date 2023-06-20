use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context, Result};
use serde::de::value::MapDeserializer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tera::{Filter, Function, Value};

fn deserialize_args<T>(args: &HashMap<String, Value>) -> Result<T>
where
    T: DeserializeOwned + Send + Sync,
{
    // https://github.com/serde-rs/serde/issues/1739
    T::deserialize(MapDeserializer::new(args.clone().into_iter()))
        .context("failed to deserialize args")
}

pub trait SimpleFunction<Params, R>
where
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    fn call(&self, args: &Params) -> Result<R>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub struct WrappedFunction<T, Params, R>
where
    T: SimpleFunction<Params, R> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    wrapped: T,
    params: PhantomData<Params>,
    result: PhantomData<R>,
}

impl<T, Params, R> Function for WrappedFunction<T, Params, R>
where
    T: SimpleFunction<Params, R> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let wrap_result = || {
            let args: Params = deserialize_args(args)?;
            let result = self.wrapped.call(&args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        };
        match wrap_result() {
            Ok(result) => Ok(result),
            Err(err) => Err(tera::Error::msg(err.to_string())),
        }
    }
    fn is_safe(&self) -> bool {
        self.wrapped.is_safe()
    }
}

pub fn wrap_function<T, Params, R>(func: T) -> WrappedFunction<T, Params, R>
where
    T: SimpleFunction<Params, R> + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    WrappedFunction {
        wrapped: func,
        params: PhantomData,
        result: PhantomData,
    }
}

pub trait SimpleFilter<V, Params, R>
where
    Params: DeserializeOwned + Send + Sync,
    V: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    fn filter(&self, value: &V, args: &Params) -> Result<R>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub struct WrappedFilter<T, V, Params, R>
where
    T: SimpleFilter<V, Params, R> + Send + Sync,
    V: DeserializeOwned + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    wrapped: T,
    value: PhantomData<V>,
    params: PhantomData<Params>,
    result: PhantomData<R>,
}

impl<T, V, Params, R> Filter for WrappedFilter<T, V, Params, R>
where
    T: SimpleFilter<V, Params, R> + Send + Sync,
    V: DeserializeOwned + Send + Sync,
    Params: DeserializeOwned + Send + Sync,
    R: Serialize + Send + Sync,
{
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let wrap_result = || {
            let value = serde_json::from_value(value.to_owned())?;
            let args: Params = deserialize_args(args)?;
            let result = self.wrapped.filter(&value, &args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        };
        match wrap_result() {
            Ok(result) => Ok(result),
            Err(err) => Err(tera::Error::msg(err.to_string())),
        }
    }
    fn is_safe(&self) -> bool {
        self.wrapped.is_safe()
    }
}
