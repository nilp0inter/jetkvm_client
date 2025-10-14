use anyhow::{Context, Result};
use dialoguer::{Input, Select, console::Term};
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::Path;

fn default_cert_path() -> String {
    "cert.pem".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JetKvmConfig {
    pub host: String,
    pub port: String,
    pub api: String,
    pub password: String,
    #[serde(default = "default_cert_path")]
    pub ca_cert_path: String,
    #[serde(default)] // Ensures `no_auto_logout` defaults to false if missing
    pub no_auto_logout: bool,
}

impl Default for JetKvmConfig {
    fn default() -> Self {
        Self {
            host: "".into(),
            port: "80".into(),
            api: "/webrtc/session".into(),
            password: "".into(),
            ca_cert_path: "cert.pem".into(),
            no_auto_logout: false,
        }
    }
}

impl JetKvmConfig {
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let toml_content = toml::to_string_pretty(&self)?;
        let mut file = fs::File::create(path)?;
        std::io::Write::write_all(&mut file, toml_content.as_bytes())?;
        Ok(())
    }
    
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let cfg = toml::from_str(&contents)?;
        Ok(cfg)
    }

        /// Attempts to load the configuration from one of the candidate files.
    /// Returns a tuple: (configuration, path that was loaded, success flag).
    /// If no configuration file is found, returns the default configuration, an empty path,
    /// and a success flag of false.
    pub fn load() -> Result<(Self, String, bool)> {
        // Define candidate configuration file locations.
        let mut config_paths = vec![
            "jetkvm_client.toml".to_string(), // local directory
        ];

        // Check Cargo project root (development mode).
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            config_paths.push(format!("{}/jetkvm_client.toml", manifest_dir));
        }

        // System-wide locations.
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        config_paths.push("/etc/jetkvm_client/jetkvm_client.toml".to_string());

        #[cfg(target_os = "windows")]
        {
            // Use ProgramData for a system-wide config on Windows.
            config_paths.push("C:\\ProgramData\\jetkvm_client\\jetkvm_client.toml".to_string());
        }

        // Search for the first candidate that exists.
        if let Some(found_path) = config_paths.iter().find(|path| Path::new(path).exists()) {
            let config_contents = fs::read_to_string(found_path)
                .context("Failed to read config file")?;
            let config: JetKvmConfig = toml::from_str(&config_contents)
                .context("Failed to parse jetkvm_client.toml")?;
            println!("✅ Loaded config from: {}", found_path);
            Ok((config, found_path.clone(), true))
        } else {
            println!("No jetkvm_client.toml found in any location.");
            Ok((JetKvmConfig::default(), "".to_string(), false))
        }
    }

    /// Returns the full URL to use for the SDP exchange.
    pub fn session_url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.api)
    }
}

/// Interactively initialize or edit the configuration file.
/// If the file exists, its current values are used as defaults.
/// Otherwise, the defaults come from `JetKvmConfig::default()`.
pub async fn interactive_config_init(path: &str) -> Result<()> {
    // Attempt to load an existing config.
    let existing_config: Option<JetKvmConfig> = fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok());
    // Use the existing config or fall back to the Default implementation.
    let defaults = existing_config.unwrap_or_default();

    println!("🛠️  Configure Your Settings");
    println!("\nThe project reads its configuration from {}.", path);

    let host: String = Input::new()
        .with_prompt("Host address")
        .default(defaults.host)
        .interact_text()?;

    let port: String = Input::new()
        .with_prompt("Port")
        .default(defaults.port)
        .interact_text()?;

    let api: String = Input::new()
        .with_prompt("API endpoint")
        .default(defaults.api)
        .interact_text()?;

    let password: String = Input::new()
        .with_prompt("Password")
        .default(defaults.password)
        .interact_text()?;

    let ca_cert_path: String = Input::new()
        .with_prompt("CA Certificate Path")
        .default(defaults.ca_cert_path)
        .interact_text()?;

    let config = JetKvmConfig {
        host,
        port,
        api,
        password,
        ca_cert_path,
        no_auto_logout: false,
    };

    config.save_to_file(path)?;
    println!("✅ Configuration saved successfully to {}.", path);
    Ok(())
}

/// Interactively choose a configuration file location for jetkvm_client.toml.
/// The candidate list is ordered such that:
///   1. CURRENT DIRECTORY has the highest precedence.
///   2. Cargo Project Root (CARGO_MANIFEST_DIR) is next.
///   3. SYSTEM-WIDE is the lowest.
/// A note is printed explaining that a configuration file in the CURRENT DIRECTORY overrides one in the Cargo Project Root,
/// which in turn overrides the SYSTEM-WIDE configuration.
/// Duplicate candidates (if the paths are the same) are merged.
// use anyhow::{Context, Result};
// use dialoguer::{Input, Select, Confirm, console::Term};
// use std::env;
// use std::fs;
use std::path::PathBuf;

pub async fn interactive_config_location() -> Result<()> {

        // Attempt to load the current configuration.
        match JetKvmConfig::load() {
            Ok((cfg, loaded_path, success)) => {
                if success {
                    println!("Current Config Loaded from: {}", loaded_path);
                } else {
                    println!("No current configuration file found; using default values.");
                }
                println!("{}", toml::to_string_pretty(&cfg)?);
            }
            Err(err) => {
                println!("Error loading configuration: {}", err);
            }
        }
        
    // Build candidate configuration file locations dynamically.
    #[cfg(not(target_os = "windows"))]
    let candidates: Vec<(String, String)> = {
        // Compute the CURRENT DIRECTORY candidate as an absolute path.
        let current_dir = env::current_dir()?;
        let current_dir_candidate: PathBuf = current_dir.join("jetkvm_client.toml");
        let current_dir_str = current_dir_candidate.canonicalize()
            .unwrap_or(current_dir_candidate.clone())
            .to_string_lossy()
            .into_owned();

        // Compute the Cargo Project Root candidate.
        let cargo_candidate_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let cargo_candidate: PathBuf = Path::new(&cargo_candidate_dir).join("jetkvm_client.toml");
        let cargo_candidate_str = cargo_candidate.canonicalize()
            .unwrap_or(cargo_candidate.clone())
            .to_string_lossy()
            .into_owned();

        // Define the SYSTEM-WIDE candidate.
        let system_candidate_str = "/etc/jetkvm_client/jetkvm_client.toml".to_string();

        // Build candidates: if CURRENT DIRECTORY equals Cargo Project Root, include only Cargo.
        let mut vec = Vec::new();
        if current_dir_str != cargo_candidate_str {
            vec.push(("Current Directory".to_string(), current_dir_str));
        }
        vec.push(("Cargo Project Root (CARGO_MANIFEST_DIR)".to_string(), cargo_candidate_str));
        vec.push(("System-wide".to_string(), system_candidate_str));
        vec
    };

    #[cfg(target_os = "windows")]
    let candidates: Vec<(String, String)> = {
        // For Windows, compute the candidates using backslashes.
        let current_dir = env::current_dir()?;
        let current_dir_candidate: PathBuf = current_dir.join("jetkvm_client.toml");
        let current_dir_str = current_dir_candidate.canonicalize()
            .unwrap_or(current_dir_candidate.clone())
            .to_string_lossy()
            .into_owned();

        let cargo_candidate_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let cargo_candidate: PathBuf = Path::new(&cargo_candidate_dir).join("jetkvm_client.toml");
        let cargo_candidate_str = cargo_candidate.canonicalize()
            .unwrap_or(cargo_candidate.clone())
            .to_string_lossy()
            .into_owned();

        // Use ProgramData for system-wide configuration on Windows.
        let system_candidate_str = "C:\\ProgramData\\jetkvm_client\\jetkvm_client.toml".to_string();

        let mut vec = Vec::new();
        if current_dir_str != cargo_candidate_str {
            vec.push(("Current Directory".to_string(), current_dir_str));
        }
        vec.push(("Cargo Project Root (CARGO_MANIFEST_DIR)".to_string(), cargo_candidate_str));
        vec.push(("System-wide".to_string(), system_candidate_str));
        vec
    };

    // (Deduplication is now implicit: if the current directory equals the cargo candidate, it was not pushed.)
    // Build dynamic precedence message:
    #[cfg(not(target_os = "windows"))]
    {
        let cargo_config = candidates
            .iter()
            .find(|(desc, _)| desc.contains("Cargo"))
            .map(|(_, path)| path)
            .unwrap_or(&"N/A".to_string())
            .clone();
        let system_config = candidates
            .iter()
            .find(|(desc, _)| desc.contains("System"))
            .map(|(_, path)| path)
            .unwrap_or(&"N/A".to_string())
            .clone();
        let current_config = candidates
            .iter()
            .find(|(desc, _)| desc.contains("Current"))
            .map(|(_, path)| path);
        println!("Configuration File Precedence:");
        if let Some(current) = current_config {
            println!("  - A configuration file in the CURRENT DIRECTORY ({})\n overrides the Cargo Project Root ({}).", current, cargo_config);
        }
        println!("  - A configuration file in the Cargo Project Root ({})\n overrides the SYSTEM-WIDE configuration ({}).", cargo_config, system_config);
        println!("(Items marked with [existing] already have a configuration file.)\n");
    }

    #[cfg(target_os = "windows")]
    {
        let cargo_config = candidates
            .iter()
            .find(|(desc, _)| desc.contains("Cargo"))
            .map(|(_, path)| path)
            .unwrap_or(&"N/A".to_string())
            .clone();
        let system_config = candidates
            .iter()
            .find(|(desc, _)| desc.contains("System"))
            .map(|(_, path)| path)
            .unwrap_or(&"N/A".to_string())
            .clone();
        if let Some(current) = candidates.iter().find(|(desc, _)| desc.contains("Current")).map(|(_, path)| path) {
            println!("Configuration File Precedence:");
            println!("  - A configuration file in the CURRENT DIRECTORY ({})\n overrides the Cargo Project Root ({}).", current, cargo_config);
        }
        println!("  - A configuration file in the Cargo Project Root ({})\n overrides the SYSTEM-WIDE configuration ({}).", cargo_config, system_config);
        println!("(Items marked with [existing] already have a configuration file.)\n");
    }

    // Build the selection list items. For each candidate, show its full absolute path.
    let items: Vec<String> = candidates
        .iter()
        .enumerate()
        .map(|(i, (desc, path_str))| {
            let flag = if Path::new(path_str).exists() {
                " [existing]"
            } else {
                ""
            };
            format!("{}: {} ({}){}", i + 1, desc, path_str, flag)
        })
        .collect();

    let selection = Select::new()
        .with_prompt("Select a configuration file location")
        .default(0)
        .items(&items)
        .interact_on_opt(&Term::stderr())?;

    if let Some(index) = selection {
        let (desc, selected_path) = &candidates[index];
        if Path::new(selected_path).exists() {
            println!(
                "You have selected an existing configuration file at {} ({}). It will be edited.",
                selected_path, desc
            );
        } else {
            println!(
                "You have selected {} ({}). A new configuration file will be created.",
                desc, selected_path
            );
        }
        // Call your interactive configuration editor.
        interactive_config_init(selected_path).await?;
    } else {
        println!("⚠️  No configuration location selected.");
    }

    Ok(())
}
