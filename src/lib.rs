#![feature(custom_derive, plugin, type_ascription, question_mark)]
#![plugin(serde_macros)]

extern crate base64;
extern crate chrono;
extern crate crypto;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate uuid;

pub mod public;
pub mod private;

pub use public::Client as PublicClient;
pub use private::Client as PrivateClient;

#[derive(Debug)]
pub enum Error {
    Api(String),
    Http(hyper::Error),
    InvalidSecretKey,
    Json(serde_json::Error),
}

impl std::convert::From<base64::Base64Error> for Error {
    fn from(_: base64::Base64Error) -> Error {
        // Only time we get a base64 error is when decoding secret key
        Error::InvalidSecretKey
    }
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
