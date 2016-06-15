#![feature(custom_derive, plugin, type_ascription, question_mark)]
#![plugin(serde_macros)]

extern crate chrono;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate uuid;

pub mod public;

pub use public::Client as PublicClient;

#[derive(Debug)]
pub enum Error {
    Api(String),
    Http(hyper::Error),
    Json(serde_json::Error)
}

impl std::convert::From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Http(err)
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}
