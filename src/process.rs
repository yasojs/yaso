use std::collections::HashMap;

use crate::module::export_default;

use rquickjs::function::Func;
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Array, BigInt, Ctx, Function, Result as QuickJsResult};

pub fn cwd() -> String {
    std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub fn get_arch() -> &'static str {
    std::env::consts::ARCH
}

pub fn get_platform() -> &'static str {
    std::env::consts::OS
}

fn hr_time_big_int(ctx: Ctx<'_>) -> QuickJsResult<BigInt> {
    let start_time = unsafe { crate::START_TIME.assume_init() };

    let elapsed = start_time.elapsed().as_nanos() as u64;

    BigInt::from_u64(ctx, elapsed)
}

fn hr_time(ctx: Ctx<'_>) -> QuickJsResult<Array<'_>> {
    let start_time = unsafe { crate::START_TIME.assume_init() };

    let elapsed = start_time.elapsed().as_nanos() as u64;

    let seconds = elapsed / 1_000_000_000;
    let remaining_nanos = elapsed % 1_000_000_000;

    let array = Array::new(ctx)?;

    array.set(0, seconds)?;
    array.set(1, remaining_nanos)?;

    Ok(array)
}

pub struct ProcessModule;

impl ModuleDef for ProcessModule {
    fn declare(declare: &mut Declarations) -> QuickJsResult<()> {
        declare.declare("argv")?;
        declare.declare("env")?;
        declare.declare("cwd")?;
        declare.declare("arch")?;
        declare.declare("platform")?;
        declare.declare("hrtime")?;
        declare.declare("exit")?;
        declare.declare("default")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> QuickJsResult<()> {
        let argv = crate::cli::get_program_argv();

        let env: HashMap<String, String> = std::env::vars().collect();

        let hrtime = Function::new(ctx.clone(), hr_time)?;
        hrtime.set("bigint", Func::from(hr_time_big_int))?;

        export_default(ctx, exports, |default| {
            default.set("argv", argv)?;
            default.set("env", env)?;
            default.set("cwd", Func::from(cwd))?;
            default.set("arch", get_arch())?;
            default.set("platform", get_platform())?;
            default.set("hrtime", hrtime)?;
            default.set(
                "exit",
                Func::from(|status_code: i32| std::process::exit(status_code)),
            )?;

            Ok(())
        })
    }
}
