use super::super::config::{HooksSection, SparkFile};
use std::process::Command;
use tracing::info;

pub fn run_pre_deploy_hooks(config: &SparkFile, app_dir: &str) {
    if let Some(hooks) = &config.hooks {
        if let Some(pre) = &hooks.pre_deploy {
            info!("Running pre-deploy hook: {}", pre);
            let _ = Command::new("sh")
                .arg("-c")
                .arg(pre)
                .current_dir(app_dir)
                .status();
        }
    }
}

pub fn run_post_deploy_hooks(config: &SparkFile, app_dir: &str) {
    if let Some(hooks) = &config.hooks {
        if let Some(post) = &hooks.post_deploy {
            info!("Running post-deploy hook: {}", post);
            let _ = Command::new("sh")
                .arg("-c")
                .arg(post)
                .current_dir(app_dir)
                .status();
        }
    }
}
