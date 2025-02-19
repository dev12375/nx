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

pub async fn run_initial_setup() -> SetupResult {
    // Get home directory and check for prover-id file
    let home_path = home::home_dir().expect("Failed to determine home directory");

    //If the .nexus directory doesn't exist, we need to create it
    let nexus_dir = home_path.join(".nexus");
    if !nexus_dir.exists() {
        create_nexus_directory(&nexus_dir).expect("Failed to create .nexus directory");
    }

    println!(
        "\n===== {} =====\n",
        "Adding your node ID to the CLI"
            .bold()
            .underline()
            .bright_cyan()
    );
    println!("You chose to start earning NEX by connecting your node ID\n");
    println!("If you don't have a node ID, you can get it by following these steps:\n");
    println!("1. Go to https://app.nexus.xyz/nodes");
    println!("2. Sign in");
    println!("3. Click on the '+ Add Node' button");
    println!("4. Select 'Add CLI node'");
    println!("5. You will be given a node ID to add to this CLI");
    println!("6. Enter the node ID into the terminal below:\n");

    let node_id = get_node_id_from_user();
    SetupResult::Connected(node_id)
}

pub fn clear_user_config() -> std::io::Result<()> {
    // Clear prover-id file
    let home_path = home::home_dir().expect("Failed to determine home directory");
    let prover_id_path = home_path.join(".nexus").join("prover-id");
    if prover_id_path.exists() {
        fs::remove_file(prover_id_path)?;
        println!("Cleared prover ID configuration");
    }

    println!("User configuration cleared");
    Ok(())
}

fn get_node_id_from_user() -> String {
    println!("{}", "Please enter your node ID:".green());
    let mut node_id = String::new();
    std::io::stdin()
        .read_line(&mut node_id)
        .expect("Failed to read node ID");
    node_id.trim().to_string()
}
