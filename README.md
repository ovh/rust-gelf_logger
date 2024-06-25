# gelf_logger 

The Graylog Extended Log Format ([GELF](http://docs.graylog.org/en/latest/pages/gelf.html)) is a log format that avoids the shortcomings of classic
log formats. GELF is a great choice for logging from within applications.
There are libraries and appenders for many programming languages and logging
frameworks, so it is easy to implement. You could use GELF to send every
exception as a log message to your Graylog cluster.

## Examples

```rust
use std::time::Duration;

use gelf_logger::{gelf_warn, GelfLevel, Builder, gelf_log, gelf_emergency, gelf_alert, gelf_critical, gelf_error, gelf_notice, gelf_info, gelf_debug};
use log::{error, info, LevelFilter, warn};
use serde::Serialize;

// Logs will be sent using a TCP socket.
Builder::new()
    .filter_level(LevelFilter::Info)
    .hostname("127.0.0.1".to_owned())
    .port(2202)
    .tls(false)
    .init();

#[derive(Serialize, Debug)]
struct Request<'a> {
    id: u16,
    method: &'a str,
    path: &'a str,
}

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
```

## License

Licensed under [BSD 3-Clause License](./LICENSE) or (https://opensource.org/licenses/BSD-3-Clause)
