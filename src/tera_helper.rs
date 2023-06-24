use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context, Result};
use serde::de::value::MapDeserializer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tera::{Filter as TeraFilter, Function as TeraFunction, Value};

fn deserialize_args<T>(args: &HashMap<String, Value>) -> Result<T>
where
    T: DeserializeOwned + Send + Sync,
{
    // https://github.com/serde-rs/serde/issues/1739
    T::deserialize(MapDeserializer::new(args.clone().into_iter()))
        .context("failed to deserialize args")
}

pub trait Function {
    type Params;
    type Result;

    fn call(&self, args: &Self::Params) -> Result<Self::Result>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub struct WrappedFunction<T> {
    wrapped: T,
}

impl<T> TeraFunction for WrappedFunction<T>
where
    T: Function + Send + Sync,
    T::Params: DeserializeOwned + Send + Sync,
    T::Result: Serialize + Send + Sync,
{
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        (|| {
            let args: T::Params = deserialize_args(args)?;
            let result = self.wrapped.call(&args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        })()
        .map_err(|err| tera::Error::chain("failed to execute tera function", err))
    }
    fn is_safe(&self) -> bool {
        self.wrapped.is_safe()
    }
}

impl<T> From<T> for WrappedFunction<T>
where
    T: Function,
{
    fn from(wrapped: T) -> Self {
        Self { wrapped }
    }
}

pub struct WrappedFnFunction<F, Params, R> {
    f: F,
    _t: PhantomData<(Params, R)>,
}

pub fn wrap_fn<F, Params, R>(f: F) -> WrappedFunction<WrappedFnFunction<F, Params, R>>
where
    F: Fn(&Params) -> Result<R>,
{
    WrappedFnFunction { f, _t: PhantomData }.into()
}

impl<F, Params, R> Function for WrappedFnFunction<F, Params, R>
where
    F: Fn(&Params) -> Result<R>,
{
    type Params = Params;
    type Result = R;

    fn call(&self, args: &Self::Params) -> Result<Self::Result> {
        (self.f)(args)
    }
}

pub trait Filter {
    type Value;
    type Params;
    type Result;

    fn filter(&self, value: &Self::Value, args: &Self::Params) -> Result<Self::Result>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub struct WrappedFilter<T> {
    wrapped: T,
}

impl<T> TeraFilter for WrappedFilter<T>
where
    T: Filter + Send + Sync,
    T::Value: DeserializeOwned + Send + Sync,
    T::Params: DeserializeOwned + Send + Sync,
    T::Result: Serialize + Send + Sync,
{
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        (|| {
            let value = serde_json::from_value(value.to_owned())?;
            let args: T::Params = deserialize_args(args)?;
            let result = self.wrapped.filter(&value, &args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        })()
        .map_err(|err| tera::Error::chain("failed to execute tera filter", err))
    }
    fn is_safe(&self) -> bool {
        self.wrapped.is_safe()
    }
}

impl<T> From<T> for WrappedFilter<T>
where
    T: Filter,
{
    fn from(wrapped: T) -> Self {
        Self { wrapped }
    }
}

pub struct WrappedFnFilter<F, V, Params, R> {
    f: F,
    _t: PhantomData<(V, Params, R)>,
}

impl<F, V, Params, R> Filter for WrappedFnFilter<F, V, Params, R>
where
    F: Fn(&V, &Params) -> Result<R>,
{
    type Value = V;
    type Params = Params;
    type Result = R;

    fn filter(&self, value: &Self::Value, args: &Self::Params) -> Result<Self::Result> {
        (self.f)(value, args)
    }
}

#[allow(dead_code)]
pub fn wrap_filter<F, V, Params, R>(f: F) -> WrappedFilter<WrappedFnFilter<F, V, Params, R>>
where
    F: Fn(&V, &Params) -> Result<R>,
{
    WrappedFnFilter { f, _t: PhantomData }.into()
}
