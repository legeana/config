use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context as _, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::de::value::MapDeserializer;
use tera::{Filter as TeraFilter, Function as TeraFunction, Value};

use crate::empty_struct;

fn deserialize_args<T>(args: &HashMap<String, Value>) -> Result<T>
where
    T: DeserializeOwned + Send + Sync,
{
    // https://github.com/serde-rs/serde/issues/1739
    T::deserialize(MapDeserializer::new(args.clone().into_iter()))
        .context("failed to deserialize args")
}

pub(crate) trait Function {
    type Params;
    type Result;

    fn call(&self, args: &Self::Params) -> Result<Self::Result>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub(crate) struct WrappedFunction<T>(T);

impl<T> TeraFunction for WrappedFunction<T>
where
    T: Function + Send + Sync,
    T::Params: DeserializeOwned + Send + Sync,
    T::Result: Serialize + Send + Sync,
{
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        (|| {
            let args: T::Params = deserialize_args(args)?;
            let result = self.0.call(&args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        })()
        .map_err(|err| tera::Error::chain("failed to execute tera function", err))
    }
    fn is_safe(&self) -> bool {
        self.0.is_safe()
    }
}

impl<T> From<T> for WrappedFunction<T>
where
    T: Function,
{
    fn from(wrapped: T) -> Self {
        Self(wrapped)
    }
}

pub(crate) struct WrappedFnFunction<F, Params, R> {
    f: F,
    _t: PhantomData<(Params, R)>,
}

pub(crate) fn wrap_fn<F, Params, R>(f: F) -> WrappedFunction<WrappedFnFunction<F, Params, R>>
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

pub(crate) struct WrappedNilFunction<F, R> {
    f: F,
    _t: PhantomData<R>,
}

#[allow(dead_code)]
pub(crate) fn wrap_nil<F, R>(f: F) -> WrappedFunction<WrappedNilFunction<F, R>>
where
    F: Fn() -> Result<R>,
{
    WrappedNilFunction { f, _t: PhantomData }.into()
}

impl<F, R> Function for WrappedNilFunction<F, R>
where
    F: Fn() -> Result<R>,
{
    type Params = empty_struct::EmptyStruct;
    type Result = R;

    fn call(&self, _args: &Self::Params) -> Result<Self::Result> {
        (self.f)()
    }
}

pub(crate) trait Filter {
    type Value;
    type Params;
    type Result;

    fn filter(&self, value: &Self::Value, args: &Self::Params) -> Result<Self::Result>;
    fn is_safe(&self) -> bool {
        false
    }
}

pub(crate) struct WrappedFilter<T>(T);

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
            let result = self.0.filter(&value, &args)?;
            serde_json::to_value(result).context("failed to serialize result to json")
        })()
        .map_err(|err| tera::Error::chain("failed to execute tera filter", err))
    }
    fn is_safe(&self) -> bool {
        self.0.is_safe()
    }
}

impl<T> From<T> for WrappedFilter<T>
where
    T: Filter,
{
    fn from(wrapped: T) -> Self {
        Self(wrapped)
    }
}

pub(crate) struct WrappedFnFilter<F, V, Params, R> {
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
pub(crate) fn wrap_filter<F, V, Params, R>(f: F) -> WrappedFilter<WrappedFnFilter<F, V, Params, R>>
where
    F: Fn(&V, &Params) -> Result<R>,
{
    WrappedFnFilter { f, _t: PhantomData }.into()
}

pub(crate) struct WrappedNilFilter<F, V, R> {
    f: F,
    _t: PhantomData<(V, R)>,
}

impl<F, V, R> Filter for WrappedNilFilter<F, V, R>
where
    F: Fn(&V) -> Result<R>,
{
    type Value = V;
    type Params = empty_struct::EmptyStruct;
    type Result = R;

    fn filter(&self, value: &Self::Value, _args: &Self::Params) -> Result<Self::Result> {
        (self.f)(value)
    }
}

#[allow(dead_code)]
pub(crate) fn wrap_nil_filter<F, V, R>(f: F) -> WrappedFilter<WrappedNilFilter<F, V, R>>
where
    F: Fn(&V) -> Result<R>,
{
    WrappedNilFilter { f, _t: PhantomData }.into()
}
