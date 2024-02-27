use crate::module::export_default;

use rquickjs::function::{Func, Rest};
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Ctx, Object, Result as QuickJsResult};

use std::path::{Component, PathBuf, MAIN_SEPARATOR_STR};

#[cfg(windows)]
const MAIN_DELIMITER_STR: &str = ";";
#[cfg(not(windows))]
const MAIN_DELIMITER_STR: &str = ":";

fn split_name_and_extension(file_name: String) -> (String, String) {
    let mut extension = PathBuf::from(&file_name)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if !extension.is_empty() {
        extension.insert(0, '.');
    }

    let name = file_name
        .get(0..(file_name.len() - extension.len()))
        .unwrap_or_default()
        .to_string();

    (name, extension)
}

fn parse(ctx: Ctx<'_>, path: String) -> QuickJsResult<Object> {
    let result = Object::new(ctx)?;

    let path_buf = PathBuf::from(path);

    let parent = path_buf
        .parent()
        .and_then(|p| p.as_os_str().to_str())
        .unwrap_or_default();

    let file_name = path_buf
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let (name, extension) = split_name_and_extension(file_name);

    let root = path_buf
        .components()
        .next()
        .and_then(|c| match c {
            Component::Prefix(prefix) => prefix.as_os_str().to_str(),
            Component::RootDir => c.as_os_str().to_str(),
            _ => Some(""),
        })
        .unwrap_or_default();

    result.set("root", root)?;
    result.set("dir", parent)?;
    result.set("base", format!("{}{}", name, extension))?;
    result.set("ext", extension)?;
    result.set("name", name)?;

    Ok(result)
}

fn format(object: Object) -> String {
    let mut result = PathBuf::new();

    let dir: String = object.get("dir").unwrap_or_default();
    let root: String = object.get("root").unwrap_or_default();
    let base: String = object.get("base").unwrap_or_default();
    let mut name: String = object.get("name").unwrap_or_default();
    let mut ext: String = object.get("ext").unwrap_or_default();

    if !dir.is_empty() {
        result.push(dir);
    } else if !root.is_empty() {
        result.push(root);
    }

    if !base.is_empty() {
        result.push(base);
    } else {
        if !ext.is_empty() {
            if !ext.starts_with('.') {
                ext.insert(0, '.');
            }

            name.push_str(&ext);
        }

        result.push(name);
    }

    result.to_string_lossy().to_string()
}

pub struct PathModule;

impl ModuleDef for PathModule {
    fn declare(declare: &mut Declarations) -> QuickJsResult<()> {
        declare.declare("parse")?;
        declare.declare("format")?;
        declare.declare("normalize")?;
        declare.declare("resolve")?;
        declare.declare("join")?;
        declare.declare("dirname")?;
        declare.declare("basename")?;
        declare.declare("extname")?;
        declare.declare("isAbsolute")?;
        declare.declare("sep")?;
        declare.declare("delimiter")?;
        declare.declare("default")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &mut Exports<'js>) -> QuickJsResult<()> {
        export_default(ctx, exports, |default| {
            default.set("parse", Func::from(parse))?;
            default.set("format", Func::from(format))?;

            default.set(
                "normalize",
                Func::from(|path: String| {
                    let mut normalized_path = PathBuf::new();

                    for component in PathBuf::from(path).components() {
                        normalized_path.push(component);
                    }

                    normalized_path.to_string_lossy().to_string()
                }),
            )?;

            default.set(
                "resolve",
                Func::from(|paths: Rest<String>| {
                    let mut final_path = PathBuf::from(crate::process::cwd());

                    for path in paths.iter() {
                        final_path.push(path);
                    }

                    final_path.to_string_lossy().to_string()
                }),
            )?;

            default.set(
                "join",
                Func::from(|paths: Rest<String>| {
                    let mut final_path = PathBuf::new();

                    for path in paths.iter() {
                        final_path.push(path.trim_start_matches('/'));
                    }

                    final_path.to_string_lossy().to_string()
                }),
            )?;

            default.set(
                "dirname",
                Func::from(|path: String| match PathBuf::from(path).parent() {
                    Some(parent) => {
                        let parent = parent.to_string_lossy().to_string();

                        if parent.is_empty() {
                            String::from(".")
                        } else {
                            parent
                        }
                    }

                    None => String::from("."),
                }),
            )?;

            default.set(
                "basename",
                Func::from(|path: String| match PathBuf::from(path).file_name() {
                    Some(file_name) => file_name.to_string_lossy().to_string(),

                    None => String::new(),
                }),
            )?;

            default.set(
                "extname",
                Func::from(|path: String| match PathBuf::from(path).extension() {
                    Some(extension) => String::from(".") + &extension.to_string_lossy(),

                    None => String::new(),
                }),
            )?;

            default.set(
                "isAbsolute",
                Func::from(|path: String| PathBuf::from(path).is_absolute()),
            )?;

            default.set("sep", MAIN_SEPARATOR_STR)?;
            default.set("delimiter", MAIN_DELIMITER_STR)?;

            Ok(())
        })
    }
}
