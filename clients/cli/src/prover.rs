use nexus_sdk::{
    stwo::seq::Stwo,
    Local,
    Prover,
    Viewable,
};

use crate::config;
use crate::flops;
use crate::orchestrator_client::OrchestratorClient;
use crate::setup;
use crate::utils;
use colored::Colorize;
use sha3::{Digest, Keccak256};
use crate::memory_stats::get_memory_info;
use std::env;
use home::home_dir;  // 确保添加 home = "0.5" 到 Cargo.toml
use std::path::{Path, PathBuf};

/// Proves a program with a given node ID
#[allow(dead_code)]
async fn authenticated_proving(
    node_id: &str,
    environment: &config::Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = OrchestratorClient::new(environment.clone());

    println!("1. Fetching a task to prove from Nexus Orchestrator...");
    let proof_task = client.get_proof_task(node_id).await?;
    println!("2. Received a task to prove from Nexus Orchestrator...");

    let public_input: u32 = proof_task.public_inputs[0] as u32;

    println!("3. Compiling guest program...");
    let elf_file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fib_input");
    let prover =
        Stwo::<Local>::new_from_file(&elf_file_path).expect("failed to load guest program");

    println!("4. Creating ZK proof with inputs...");
    let (view, proof) = prover
        .prove_with_input::<(), u32>(&(), &public_input)
        .expect("Failed to run prover");

    assert_eq!(view.exit_code().expect("failed to retrieve exit code"), 0);

    let proof_bytes = serde_json::to_vec(&proof)?;
    let proof_hash = format!("{:x}", Keccak256::digest(&proof_bytes));

    println!("\tProof size: {} bytes", proof_bytes.len());
    println!("5. Submitting ZK proof to Nexus Orchestrator...");
    client
        .submit_proof(node_id, &proof_hash, proof_bytes)
        .await?;
    println!("{}", "6. ZK proof successfully submitted".green());

    Ok(())
}

// 添加新的结构体来存储配置
pub struct ProverConfig {
    pub elf_file_path: String,
}

impl Default for ProverConfig {
    fn default() -> Self {
        let home_path = home_dir()
            .unwrap_or_else(|| PathBuf::from("/root"));  // 如果找不到主目录就用 /root
            
        let file_path = home_path
            .join(".nexus")
            .join("network-api")
            .join("clients")
            .join("cli")
            .join("assets")
            .join("fib_input");
            
        Self {
            elf_file_path: file_path.to_string_lossy().to_string(),
        }
    }
}

// 修改函数签名，添加配置参数
fn anonymous_proving(config: &ProverConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 获取系统内存信息 (MiB)
    let (used_mem, total_mem) = get_memory_info();
    let available_mem = total_mem - used_mem;
    
    // 根据总可用内存大小选择合适的输入值 (MiB)
    let public_input: u32 = if available_mem < 512 {      // 小于 512MB
        1  // 最小输入值
    } else if available_mem < 1024 {  // 小于 1GB
        2  // 较小输入值
    } else if available_mem < 2048 {  // 小于 2GB
        3  // 中等输入值
    } else {
        4  // 标准输入值
    };

    println!("System Memory - Total: {}MiB, Used: {}MiB, Available: {}MiB", 
        total_mem,
        used_mem,
        available_mem,
    );
    println!("Selected input value {} for system with {}MiB available memory", 
        public_input,
        available_mem
    );

    println!("1. Compiling guest program...");
    let elf_file_path = std::path::Path::new(&config.elf_file_path);
    
    println!("Looking for file at: {}", elf_file_path.display());
    
    if !elf_file_path.exists() {
        return Err(format!("File not found at: {}", elf_file_path.display()).into());
    }

    let prover = match Stwo::<Local>::new_from_file(&elf_file_path) {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to load guest program: {}", e).into())
    };

    //3. Run the prover
    println!("2. Creating ZK proof...");
    let (view, proof) = match prover.prove_with_input::<(), u32>(&(), &public_input) {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Failed to create proof (memory error?): {}", e).into());
        }
    };

    assert_eq!(view.exit_code().expect("failed to retrieve exit code"), 0);

    let proof_bytes = serde_json::to_vec(&proof)?;

    println!(
        "{}",
        format!(
            "3. ZK proof successfully created with size: {} bytes",
            proof_bytes.len()
        )
        .green(),
    );
    Ok(())
}

/// Starts the prover, which can be anonymous or connected to the Nexus Orchestrator
pub async fn start_prover(
    environment: &config::Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print the banner at startup
    utils::cli_branding::print_banner();

    println!(
        "\n===== {} =====\n",
        "Setting up CLI configuration"
            .bold()
            .underline()
            .bright_cyan(),
    );

    // Run the initial setup to determine anonymous or connected node
    match setup::run_initial_setup().await {
        setup::SetupResult::Anonymous => {
            println!(
                "\n===== {} =====\n",
                "Starting Anonymous proof generation for programs"
                    .bold()
                    .underline()
                    .bright_cyan()
            );
            
            // 使用默认配置或从环境变量获取
            let config = ProverConfig {
                elf_file_path: std::env::var("PROVER_ELF_PATH")
                    .unwrap_or_else(|_| ProverConfig::default().elf_file_path),
            };

            let mut proof_count = 1;
            loop {
                println!("\n================================================");
                println!(
                    "{}",
                    format!("\nStarting proof #{} ...\n", proof_count).yellow()
                );
                match anonymous_proving(&config) {
                    Ok(_) => (),
                    Err(e) => println!("Error in anonymous proving: {}", e),
                }
                proof_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            }
        }
        setup::SetupResult::Connected(node_id) => {
            println!(
                "\n===== {} =====\n",
                "Starting proof generation for programs"
                    .bold()
                    .underline()
                    .bright_cyan()
            );
            let flops = flops::measure_flops();
            let flops_formatted = format!("{:.2}", flops);
            let flops_str = format!("{} FLOPS", flops_formatted);
            println!(
                "{}: {}",
                "Computational capacity of this node".bold(),
                flops_str.bright_cyan()
            );
            println!(
                "{}: {}",
                "You are proving with node ID".bold(),
                node_id.bright_cyan()
            );
            println!(
                "{}: {}",
                "Environment".bold(),
                environment.to_string().bright_cyan()
            );

            let mut proof_count = 1;
            loop {
                println!("\n================================================");
                println!(
                    "{}",
                    format!("\nStarting proof #{} ...\n", proof_count).yellow()
                );

                match anonymous_proving(&ProverConfig::default()) {
                    Ok(_) => (),
                    Err(e) => println!("Error in anonymous proving: {}", e),
                }
                proof_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            }
        }
        setup::SetupResult::Invalid => Err("Invalid setup option selected".into()),
    }
}
