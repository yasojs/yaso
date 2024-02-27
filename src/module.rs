use rquickjs::module::Exports;
use rquickjs::{Ctx, Object, Result as QuickJsResult, Value};

pub fn export_default<'js, F>(ctx: &Ctx<'js>, exports: &mut Exports<'js>, f: F) -> QuickJsResult<()>
where
    F: FnOnce(&Object<'js>) -> QuickJsResult<()>,
{
    let default = Object::new(ctx.clone())?;

    f(&default)?;

    for name in default.keys::<String>() {
        let name = name?;
        let value: Value = default.get(name.clone())?;

        exports.export(name, value)?;
    }

    exports.export("default", default)?;

    Ok(())
}
