#![feature(
    async_await,
    await_macro,
    futures_api,
    pin,
    try_trait,
    underscore_imports,
)]

#[macro_use] extern crate futures;
#[macro_use] extern crate redis_async;

pub mod model;

mod cacher;
mod error;
mod gen;

pub use crate::{
    cacher::DabbotCache,
    error::{Error, FutureResult},
};
