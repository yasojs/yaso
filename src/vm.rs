use crate::os::OsModule;

use rquickjs::loader::{BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Module, Object, Value,
};

use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::exit;

macro_rules! create_modules {
    ($($name:expr => $module:expr),*) => {
        pub fn create_module_instances() -> (BuiltinResolver, ModuleLoader) {
            let mut builtin_resolver = BuiltinResolver::default();
            let mut module_loader = ModuleLoader::default();

            $(
                builtin_resolver = builtin_resolver.with_module($name);
                module_loader = module_loader.with_module($name, $module);
            )*

            (builtin_resolver, module_loader)
        }
    };
}

create_modules!(
    "os" => OsModule
);

pub struct VirtualMachine {
    context: AsyncContext,
    runtime: AsyncRuntime,
}

impl VirtualMachine {
    pub async fn new() -> VirtualMachine {
        let (builtin_resolver, module_loader) = create_module_instances();

        let resolver = (
            builtin_resolver,
            FileResolver::default()
                .with_path(".")
                .with_pattern("{}.cjs")
                .with_pattern("{}.mjs"),
        );

        let loader = (
            module_loader,
            ScriptLoader::default()
                .with_extension("mjs")
                .with_extension("cjs"),
        );

        let runtime = AsyncRuntime::new().expect("failed to create an AsyncRuntime");

        runtime.set_loader(resolver, loader).await;

        let context = AsyncContext::full(&runtime)
            .await
            .expect("failed to create an AsyncContext");

        VirtualMachine { context, runtime }
    }

    pub async fn init(&self) {
        self.context
            .with(|ctx| {
                crate::console::init(&ctx)
                    .catch(&ctx)
                    .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx, err));
            })
            .await
    }

    pub async fn repl(&self) {
        loop {
            print!(">> ");
            stdout().flush().unwrap();

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            self.context
                .with(|ctx| {
                    match ctx.eval::<Value, String>(input).catch(&ctx) {
                        Ok(value) => match crate::console::js_stringify(&value).catch(&ctx) {
                            Ok(value) => {
                                println!("{}", value);
                            }

                            Err(err) => VirtualMachine::print_error(ctx, err),
                        },

                        Err(err) => VirtualMachine::print_error(ctx, err),
                    };
                })
                .await;
        }
    }

    pub async fn idle(self) {
        self.runtime.idle().await;

        drop(self.context);
        drop(self.runtime);
    }

    fn load_module<'js>(ctx: &Ctx<'js>, file_path: &Path) -> Result<Object<'js>, rquickjs::Error> {
        Module::import(ctx, file_path.to_string_lossy().to_string())
    }

    pub async fn run_module(&self, file_path: &Path) {
        self.context
            .with(|ctx| {
                VirtualMachine::load_module(&ctx, file_path)
                    .catch(&ctx)
                    .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx, err));
            })
            .await
    }

    fn print_error<'js>(ctx: Ctx<'js>, err: CaughtError<'js>) {
        let error_message = match err {
            CaughtError::Error(err) => err.to_string(),

            CaughtError::Exception(exception) => crate::console::js_stringify(exception.as_value())
                .catch(&ctx)
                .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx, err)),

            CaughtError::Value(value) => crate::console::js_stringify(&value)
                .catch(&ctx)
                .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx, err)),
        };

        eprintln!("{}", error_message);
    }

    fn print_error_and_exit<'js>(ctx: Ctx<'js>, err: CaughtError<'js>) -> ! {
        VirtualMachine::print_error(ctx, err);

        exit(1);
    }
}
