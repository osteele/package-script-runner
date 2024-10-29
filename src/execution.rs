use std::collections::HashMap;
use anyhow::Result;

use anyhow::Context;

use crate::package_managers::PackageManager;

pub fn run_script(
    package_manager: &Box<dyn PackageManager>,
    script: &str,
    args: &[String],
) -> Result<i32> {
    run_script_with_env(package_manager, script, args, &HashMap::new())
}

pub fn run_script_with_env(
    package_manager: &Box<dyn PackageManager>,
    script: &str,
    args: &[String],
    env_vars: &HashMap<String, String>,
) -> Result<i32> {
    let mut command = package_manager.run_command(script);
    command.args(args);
    command.envs(env_vars);

    let status = command.status().context("Failed to run script")?;

    Ok(status.code().unwrap_or(-1))
}
