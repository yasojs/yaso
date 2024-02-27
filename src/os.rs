use crate::module::export_default;

use rquickjs::function::Func;
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Ctx, Result as QuickJsResult};

use libc::utsname;

use once_cell::sync::Lazy;

use std::ffi::{c_char, CStr};
use std::io;

#[derive(Debug, Clone)]
struct Uname {
    sysname: String,
    release: String,
    version: String,
    machine: String,
}

impl Uname {
    pub fn new() -> io::Result<Uname> {
        let mut name = unsafe { std::mem::zeroed() };

        let result = unsafe { libc::uname(&mut name) };

        if result == 0 {
            Ok(Uname::from(name))
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl From<utsname> for Uname {
    fn from(x: utsname) -> Self {
        fn to_cstr(buf: &[c_char]) -> &CStr {
            unsafe { CStr::from_ptr(buf.as_ptr()) }
        }

        Uname {
            sysname: to_cstr(&x.sysname).to_string_lossy().to_string(),
            release: to_cstr(&x.release).to_string_lossy().to_string(),
            version: to_cstr(&x.version).to_string_lossy().to_string(),
            machine: to_cstr(&x.machine).to_string_lossy().to_string(),
        }
    }
}

static OS_INFO: Lazy<(String, String, String, String)> = Lazy::new(|| match Uname::new() {
    Ok(uname) => (uname.sysname, uname.release, uname.version, uname.machine),

    Err(_) => (
        String::from("N/A"),
        String::from("N/A"),
        String::from("N/A"),
        String::from("N/A"),
    ),
});

fn get_type() -> &'static str {
    &OS_INFO.0
}

fn get_release() -> &'static str {
    &OS_INFO.1
}

fn get_version() -> &'static str {
    &OS_INFO.2
}

fn get_machine() -> &'static str {
    &OS_INFO.3
}

fn get_tmp_dir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

pub struct OsModule;

impl ModuleDef for OsModule {
    fn declare(declare: &mut Declarations) -> QuickJsResult<()> {
        declare.declare("type")?;
        declare.declare("release")?;
        declare.declare("version")?;
        declare.declare("machine")?;
        declare.declare("arch")?;
        declare.declare("platform")?;
        declare.declare("tmpdir")?;
        declare.declare("default")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> QuickJsResult<()> {
        export_default(ctx, exports, |default| {
            default.set("type", Func::from(get_type))?;
            default.set("release", Func::from(get_release))?;
            default.set("version", Func::from(get_version))?;
            default.set("machine", Func::from(get_machine))?;
            default.set("arch", Func::from(crate::process::get_arch))?;
            default.set("platform", Func::from(crate::process::get_platform))?;
            default.set("tmpdir", Func::from(get_tmp_dir))?;

            Ok(())
        })
    }
}
