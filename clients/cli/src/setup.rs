use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;

// Update the import path to use the proto module
use crate::node_id_manager::{
    create_nexus_directory, get_home_directory, handle_read_error, read_existing_node_id,
};

pub enum SetupResult {
    Anonymous,
    Connected(String),
    Invalid,
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    pub node_id: String,
    pub user_id: Option<String>,
}

//function that takes a node_id and saves it to the user config
fn save_node_id(node_id: &str) -> std::io::Result<()> {
    //get the home directory
    let home_path = match get_home_directory() {
        Ok(path) => path,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to determine home directory",
            ))
        }
    };

    let nexus_dir = home_path.join(".nexus");
    let node_id_path = nexus_dir.join("node-id");

    //print how to find the node-id file
    println!("Node ID file: {}", node_id_path.to_string_lossy());
    //write the node_id to the node-id file
    fs::write(&node_id_path, node_id).unwrap();

    // 2. If the .nexus directory exists, we need to read the node-id file
    match read_existing_node_id(&node_id_path) {
        // 2.1 Happy path - we successfully read the node-id file
        Ok(id) => {
            println!(
                "Successfully read existing node-id '{}' from file: {}",
                id,
                node_id_path.to_string_lossy()
            );
            Ok(())
        }
        // 2.2 We couldn't read the node-id file, so we may need to create a new one
        Err(e) => {
            eprintln!(
                "{}: {}",
                "Warning: Could not read node-id file".to_string().yellow(),
                e
            );
            handle_read_error(e, &node_id_path, node_id);
            Ok(())
        }
    }
}

pub async fn run_initial_setup(node_id: Option<String>) -> SetupResult {
    if let Some(node_id) = node_id {
        // 直接返回 Connected 结果，不保存文件
        SetupResult::Connected(node_id)
    } else {
        // 原有的交互式流程
        println!("\nThis node is not connected to any account.\n");
        println!("Please enter your node ID:");
        let node_id = get_node_id_from_user();
        SetupResult::Connected(node_id)
    }
}

pub fn clear_node_id() -> std::io::Result<()> {
    let home_path: std::path::PathBuf =
        home::home_dir().expect("Failed to determine home directory");

    //If the .nexus directory doesn't exist, nothing to clear
    let nexus_dir = home_path.join(".nexus");
    if !nexus_dir.exists() {
        // nothing to clear
        return Ok(());
    }

    // if the nexus directory exists, check if the node-id file exists
    let node_id_path = home_path.join(".nexus").join("node-id");
    if !node_id_path.exists() {
        // nothing to clear
        return Ok(());
    }

    //if the node-id file exists, clear it
    match fs::remove_file(&node_id_path) {
        Ok(_) => {
            println!(
                "Successfully cleared node ID configuration with file: {}",
                node_id_path.to_string_lossy()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Failed to clear node ID configuration: {}", e).red()
            );
            Err(e)
        }
    }
}

fn get_node_id_from_user() -> String {
    println!("{}", "Please enter your node ID:".green());
    let mut node_id = String::new();
    std::io::stdin()
        .read_line(&mut node_id)
        .expect("Failed to read node ID");
    node_id.trim().to_string()
}
