mod go;
mod node;
mod python;
mod rust;

use anyhow::Result;
use std::{path::Path, process::Command};

use crate::script_type::Script;

use go::GoPackageManager;
use node::NodePackageManager;
use python::PythonPackageManager;
use rust::RustPackageManager;

pub trait PackageManager {
    fn detect(dir: &Path) -> Option<Self>
    where
        Self: Sized;
    fn run_command(&self, script: &str) -> Command;
    fn parse_scripts(&self, path: &Path) -> Result<Vec<Script>>;
}

pub fn detect_package_manager_in_dir(dir: &Path) -> Option<Box<dyn PackageManager>> {
    if let Some(npm) = NodePackageManager::detect(dir) {
        Some(Box::new(npm))
    } else if let Some(rust) = RustPackageManager::detect(dir) {
        Some(Box::new(rust))
    } else if let Some(python) = PythonPackageManager::detect(dir) {
        Some(Box::new(python))
    } else if let Some(go) = GoPackageManager::detect(dir) {
        Some(Box::new(go))
    } else {
        None
    }
}
