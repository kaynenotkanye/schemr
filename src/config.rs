use anyhow::{Context};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

const CONFIG_FILE: &str = "schemr.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password_env: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub environments: HashMap<String, Environment>,
}

impl Config {
    /// Load configuration from schemr.toml
    pub fn load() -> anyhow::Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE)
            .context("Failed to read schemr.toml; please run `schemr configure` first.")?;
        let cfg: Config = toml::from_str(&content)
            .context("Failed to parse schemr.toml")?;
        Ok(cfg)
    }

    /// Save configuration back to schemr.toml
    pub fn save(&self) -> anyhow::Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(CONFIG_FILE)?;
        file.write_all(toml_str.as_bytes())?;
        Ok(())
    }

    /// Interactive setup for environments
    pub fn configure() -> anyhow::Result<()> {
        let theme = ColorfulTheme::default();
        let mut cfg = if Path::new(CONFIG_FILE).exists() {
            Config::load().unwrap_or_default()
        } else {
            Config::default()
        };

        loop {
            println!("🔧 Add or update an environment");
            let env_name: String = Input::with_theme(&theme)
                .with_prompt("Environment name (e.g., qa, prod)")
                .interact_text()?;

            let host: String = Input::with_theme(&theme)
                .with_prompt(format!("Host for '{}'", env_name))
                .default("localhost".into())
                .interact_text()?;

            let port: u16 = Input::with_theme(&theme)
                .with_prompt(format!("Port for '{}'", env_name))
                .default(3306)
                .interact_text()?;

            let username: String = Input::with_theme(&theme)
                .with_prompt(format!("Username for '{}'", env_name))
                .default("root".into())
                .interact_text()?;

            let password_env: String = Input::with_theme(&theme)
                .with_prompt(format!("Name of ENV var for password for '{}'", env_name))
                .interact_text()?;

            let database: String = Input::with_theme(&theme)
                .with_prompt(format!("Database name for '{}'", env_name))
                .interact_text()?;

            cfg.environments.insert(
                env_name.clone(),
                Environment { host, port, username, password_env, database },
            );
            println!("✅ Environment '{}' configured", env_name);

            let again = Confirm::with_theme(&theme)
                .with_prompt("Add or update another environment?")
                .default(false)
                .interact()?;
            if !again {
                break;
            }
        }

        cfg.save()?;
        println!("✅ Configuration saved to {}", CONFIG_FILE);
        Ok(())
    }

    /// Retrieve a configured environment or error
    pub fn get_env(&self, name: &str) -> anyhow::Result<&Environment> {
        self.environments.get(name)
            .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found in {}", name, CONFIG_FILE))
    }
}
