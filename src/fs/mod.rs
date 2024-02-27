mod file;

use crate::module::export_default;

use rquickjs::function::{Async, Func, Opt};
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Class, Ctx, Exception, Result as QuickJsResult};

async fn open(ctx: Ctx<'_>, path: String, flags: Opt<String>) -> QuickJsResult<file::File> {
    let flags = flags.0.unwrap_or(String::from("r"));

    let mut open_options = tokio::fs::OpenOptions::new();

    for flag in flags.chars() {
        match flag {
            'r' => {
                open_options = open_options.read(true).clone();
            }

            'w' => {
                open_options = open_options.write(true).clone();
            }

            'c' => {
                open_options = open_options.append(true).clone();
            }

            _ => {
                return Err(Exception::throw_message(
                    &ctx,
                    &format!("Invalid flag: {}", flag),
                ))
            }
        }
    }

    match open_options.open(path).await {
        Ok(inner) => Ok(file::File::new(inner)),

        Err(err) => Err(Exception::throw_message(
            &ctx,
            &format!("Could not open file: {}", err),
        )),
    }
}

fn open_sync(ctx: Ctx<'_>, path: String, flags: Opt<String>) -> QuickJsResult<file::FileSync> {
    let flags = flags.0.unwrap_or(String::from("r"));

    let mut open_options = std::fs::OpenOptions::new();

    for flag in flags.chars() {
        match flag {
            'r' => {
                open_options = open_options.read(true).clone();
            }

            'w' => {
                open_options = open_options.write(true).clone();
            }

            'c' => {
                open_options = open_options.append(true).clone();
            }

            _ => {
                return Err(Exception::throw_message(
                    &ctx,
                    &format!("Invalid flag: {}", flag),
                ))
            }
        }
    }

    match open_options.open(path) {
        Ok(inner) => Ok(file::FileSync::new(inner)),

        Err(err) => Err(Exception::throw_message(
            &ctx,
            &format!("Could not open file: {}", err),
        )),
    }
}

pub struct FsModule;

impl ModuleDef for FsModule {
    fn declare(declare: &mut Declarations) -> QuickJsResult<()> {
        declare.declare("open")?;
        declare.declare("openSync")?;
        declare.declare("default")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> QuickJsResult<()> {
        Class::<file::File>::register(ctx)?;

        export_default(ctx, exports, |default| {
            default.set("open", Func::from(Async(open)))?;
            default.set("openSync", Func::from(open_sync))?;

            Ok(())
        })
    }
}
