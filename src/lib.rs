//! MessagePack body parsing for fornthulani.
//!
//! Adds a single method to [`fornthulani::Request`]:
//!
//! ```no_run
//! use fornthulani::{Params, Request, Response, Status, ThulaniError};
//! use fornthulani_plugin_msgpack_body::MsgPackBody;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct NewUser { name: String }
//!
//! fn create(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
//!     let input: NewUser = req.msgpack()?;
//!     Ok(Response::json(Status::CREATED, &input.name))
//! }
//! ```

use fornthulani::{Request, ThulaniError};

/// Extension trait adding `.msgpack::<T>()` to `fornthulani::Request`.
pub trait MsgPackBody {
    fn msgpack<T: serde::de::DeserializeOwned>(&self) -> Result<T, ThulaniError>;
}

impl MsgPackBody for Request {
    fn msgpack<T: serde::de::DeserializeOwned>(&self) -> Result<T, ThulaniError> {
        // 1. content-type check
        let ct = self.content_type().unwrap_or("");
        if !ct.starts_with("application/msgpack") && !ct.starts_with("application/x-msgpack") {
            return Err(ThulaniError::BadRequest(format!(
                "expected Content-Type: application/msgpack, got `{ct}`"
            )));
        }

        // 2. empty body
        if self.body.is_empty() {
            return Err(ThulaniError::BadRequest("empty body".into()));
        }

        // 3. parse
        rmp_serde::from_slice(&self.body)
            .map_err(|e| ThulaniError::BadRequest(format!("invalid msgpack: {e}")))
    }
}
