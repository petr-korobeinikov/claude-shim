use std::ffi::OsStr;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    if invoked_as_shim() {
        return claudectl::shim::run();
    }
    claudectl::run()
}

fn invoked_as_shim() -> bool {
    std::env::args_os()
        .next()
        .as_deref()
        .map(Path::new)
        .and_then(Path::file_name)
        == Some(OsStr::new("claude"))
}
