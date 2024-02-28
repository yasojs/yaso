use crate::events::EventsModule;
use crate::fs::FsModule;
use crate::os::OsModule;
use crate::path::PathModule;
use crate::process::ProcessModule;

use rquickjs::function::Func;
use rquickjs::loader::{BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader};
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Module, Object,
    Result as QuickJsResult, Value,
};

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
    "events" => EventsModule,
    "fs" => FsModule,
    "os" => OsModule,
    "path" => PathModule,
    "process" => ProcessModule
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

        let runtime = AsyncRuntime::new().expect("Could not create an AsyncRuntime");

        runtime.set_loader(resolver, loader).await;

        let context = AsyncContext::full(&runtime)
            .await
            .expect("Could not create an AsyncContext");

        VirtualMachine { context, runtime }
    }

    pub async fn init(&self) {
        self.context
            .with(|ctx| {
                let globals = ctx.globals();

                globals
                    .set(
                        "require",
                        Func::from(|ctx, specifier: String| -> QuickJsResult<Value> {
                            let module = Module::import::<Object, _>(&ctx, specifier)?;

                            Ok(module.get("default").unwrap_or(module.into_value()))
                        }),
                    )
                    .catch(&ctx)
                    .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx.clone(), err));

                crate::console::init(&ctx)
                    .catch(&ctx)
                    .unwrap_or_else(|err| VirtualMachine::print_error_and_exit(ctx.clone(), err));
            })
            .await
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

    fn print_error_and_exit<'js>(ctx: Ctx<'js>, err: CaughtError<'js>) -> ! {
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

        exit(1);
    }
}
