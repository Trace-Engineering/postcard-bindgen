use core::borrow::Borrow;
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use postcard_bindgen_core::{
    code_gen::dart::{generate, GenerationSettings},
    registry::ContainerCollection,
};

use super::PackageInfo;

/// Build a dependency-free Dart/Flutter package.
pub fn build_dart_package(
    parent_dir: &Path,
    package_info: PackageInfo,
    settings: impl Borrow<GenerationSettings>,
    bindings: ContainerCollection,
) -> io::Result<()> {
    let package_name = package_info.name.replace('-', "_");
    let dir = parent_dir.join(&package_name);
    let lib = dir.join("lib");
    std::fs::create_dir_all(&lib)?;

    let pubspec = format!(
        "name: {package_name}\nversion: {}\ndescription: Auto-generated postcard bindings for Dart and Flutter.\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n",
        package_info.version
    );
    File::create(dir.join("pubspec.yaml"))?.write_all(pubspec.as_bytes())?;
    File::create(lib.join(format!("{package_name}.dart")))?
        .write_all(generate(bindings, settings.borrow()).as_bytes())?;
    Ok(())
}
