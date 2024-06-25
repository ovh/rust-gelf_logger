// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

//! The Graylog Extended Log Format ([GELF](http://docs.graylog.org/en/latest/pages/gelf.html)) is a log format that avoids the shortcomings of classic
//! log formats. GELF is a great choice for logging from within applications.
//! There are libraries and appenders for many programming languages and logging
//! frameworks, so it is easy to implement. You could use GELF to send every
//! exception as a log message to your Graylog cluster.
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::time::Duration;
//!
//! use gelf_logger::{gelf_warn, GelfLevel, Builder, gelf_log, gelf_emergency, gelf_alert, gelf_critical, gelf_error, gelf_notice, gelf_info, gelf_debug};
//! use log::{error, info, LevelFilter, warn};
//! use serde::Serialize;
//!
//! // Logs will be sent using a TCP socket.
//! Builder::new()
//!     .filter_level(LevelFilter::Info)
//!     .hostname("127.0.0.1".to_owned())
//!     .port(2202)
//!     .tls(false)
//!     .init();
//!
//! #[derive(Serialize, Debug)]
//! struct Request<'a> {
//!     id: u16,
//!     method: &'a str,
//!     path: &'a str,
//! }
//!
//! // Basic kv logs.
//! info!(count = 5; "packet received");
//! warn!(user = "foo"; "unknown user");
//! error!(err:err = "abc".parse::<u32>().unwrap_err(); "parse error");
//!
//! let req = Request {
//!     id: 42,
//!     method: "GET",
//!     path: "/login",
//! };
//! // Will serialize as a `Debug` string.
//! info!(req:?; "incoming request");
//! // Will flatten all the field and add them as additional fields.
//! info!(req:serde; "incoming request");
//!
//! // Gelf specific levels.
//! gelf_log!(GelfLevel::Emergency, foo = "bar"; "an emergency log");
//! gelf_emergency!(foo = "bar"; "an emergency log");
//! gelf_alert!(foo = "bar"; "an alert log");
//! gelf_critical!(foo = "bar"; "a critical log");
//! gelf_error!(foo = "bar"; "an error log");
//! gelf_warn!(foo = "bar"; "a warn log");
//! gelf_notice!(foo = "bar"; "a notice log");
//! gelf_info!(foo = "bar"; "an info log");
//! gelf_debug!(foo = "bar"; "a debug log");
//!
//! // Flush underlying TCP socket.
//! // This will only flush. The socket may be dropped without proper closing.
//! log::logger().flush();
//! ```
#![doc(
    html_logo_url = "https://eu.api.ovh.com/images/com-square-bichro.png",
    html_favicon_url = "https://www.ovh.com/favicon.ico"
)]
#![warn(
    clippy::all,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

mod builder;
mod error;
mod level;
mod logger;
mod macros;
mod record;

pub use builder::Builder;
pub use error::Error;
pub use level::GelfLevel;
pub use logger::{GelfLogger, Target, TcpTarget};
pub use record::GelfRecord;
#[doc(hidden)]
pub use record::INTERNAL_LEVEL_FIELD_NAME;
#[doc(no_inline)]
pub use serde_json::Value;
