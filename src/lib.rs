#![feature(
    async_await,
    await_macro,
    decl_macro,
    futures_api,
    pin,
    try_trait,
    underscore_imports,
)]

#[macro_use] extern crate log;
#[macro_use] extern crate redis_async;
#[macro_use] extern crate serde;

pub mod model;

mod cache;
mod error;
mod gen;
mod resp_impl;

pub use crate::{
    cache::Cache,
    error::{Error, Result},
};
