use std::path::Path;
use std::path::PathBuf;

use crate::package_managers::detect_package_manager_in_dir;
use crate::package_managers::PackageManager;
use crate::types::Script;
use anyhow::Result;

pub struct Project {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub path: PathBuf,
    pub package_manager: Box<dyn PackageManager>,
}

impl Project {
    pub fn scripts(&self) -> Result<Vec<Script>> {
        self.package_manager.find_scripts(&self.path)
    }

    pub fn detect(path: &Path) -> Option<Project> {
        detect_project(path)
    }

    pub fn create(name: &str, path: &Path) -> Option<Project> {
        create_project(name, path)
    }
}

fn search_upwards_for_package_manager(dir: &Path) -> Option<(Box<dyn PackageManager>, PathBuf)> {
    let mut current_dir = dir;
    let home_dir = dirs::home_dir()?;

    while current_dir >= home_dir.as_path() {
        if let Some(pm) = detect_package_manager_in_dir(current_dir) {
            return Some((pm, current_dir.to_path_buf()));
        }
        current_dir = current_dir.parent()?;
    }

    None
}

pub fn detect_project(dir: &Path) -> Option<Project> {
    let (pm, path) = search_upwards_for_package_manager(dir)?;
    Some(Project {
        name: Some(path.to_string_lossy().to_string()),
        path: path,
        package_manager: pm,
    })
}

pub fn create_project(name: &str, path: &Path) -> Option<Project> {
    if let Some(pm) = detect_package_manager_in_dir(path) {
        Some(Project {
            name: Some(name.to_string()),
            path: path.to_path_buf(),
            package_manager: pm,
        })
    } else {
        None
    }
}
