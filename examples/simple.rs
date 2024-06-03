use std::env;

use gelf_logger::{
    gelf_alert, gelf_critical, gelf_debug, gelf_emergency, gelf_error, gelf_info, gelf_log,
    gelf_notice, gelf_warn, Builder, GelfLevel,
};
use log::{error, info, warn};
use serde::Serialize;
use serde_json::Value;

fn main() {
    // Init logger.
    let mut builder = Builder::new()
        .parse_filters("info")
        .extend_additional_fields([(
            "instance".to_owned(),
            Value::String("instance-1".to_owned()),
        )]);
    builder = match env::args().nth(1).unwrap_or("stderr".to_owned()).as_str() {
        "stdout" => builder.stdout(),
        "stderr" => builder.stderr(),
        endpoint => {
            let (hostname, port) = endpoint.split_once(':').expect("invalid tcp endpoint");
            builder
                .hostname(hostname.to_owned())
                .port(port.parse().expect("invalid port"))
        }
    };
    builder.init();

    // Basic kv logs.
    info!(count = 5; "packet received");
    warn!(user = "foo"; "unknown user");
    error!(err:err = "abc".parse::<u32>().unwrap_err(); "parse error");

    let req = Request {
        id: 42,
        method: "GET",
        path: "/login",
    };
    // Will serialize as a `Debug` string.
    info!(req:?; "incoming request");
    // Will flatten all the field and add them as additional fields.
    info!(req:serde; "incoming request");

    // Gelf specific levels.
    gelf_log!(GelfLevel::Emergency, foo = "bar"; "an emergency log");
    gelf_emergency!(foo = "bar"; "an emergency log");
    gelf_alert!(foo = "bar"; "an alert log");
    gelf_critical!(foo = "bar"; "a critical log");
    gelf_error!(foo = "bar"; "an error log");
    gelf_warn!(foo = "bar"; "a warn log");
    gelf_notice!(foo = "bar"; "a notice log");
    gelf_info!(foo = "bar"; "an info log");
    gelf_debug!(foo = "bar"; "a debug log");

    // Flush underlying TCP socket.
    // This will only flush. The socket may be dropped without proper closing.
    log::logger().flush();
}

#[derive(Serialize, Debug)]
struct Request<'a> {
    id: u16,
    method: &'a str,
    path: &'a str,
}
