use anyhow::Result;
use ethers::signers::{LocalWallet, Wallet};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
}

#[derive(Serialize)]
pub struct Field {
    pub name: String,
    pub max_length: usize,
}

pub fn deploy_contract(config: Config) -> Result<String> {
    let wallet: LocalWallet = config.chain.private_key.into();
    let provider = Provider::<Http>::try_from(rpc_url.as_str())?
        .interval(std::time::Duration::from_millis(10u64));

    // Compile the contract
}

pub fn create_contract(contract_data: ContractData) -> Result<()> {
    // Initialize Handlebars
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("contract", include_str!("../templates/contract.hbs"))?;

    // Render the template with the data
    let rendered = handlebars.render("contract", &contract_data)?;

    // Write the rendered template to a file
    std::fs::write("contract.sol", rendered)?;

    Ok(())
}

// Function to compile the Solidity contract
fn compile_contract(contract_path: &str) -> eyre::Result<artifacts::CompiledContract> {
    use ethers::solc::{Project, ProjectPathsConfig};

    let paths = ProjectPathsConfig::builder()
        .root("./")
        .sources("./") // Assuming contract.sol is in the root
        .build()?;

    let project = Project::builder()
        .paths(paths)
        .ephemeral() // Do not write artifacts to disk
        .no_artifacts()
        .build()?;

    // Compile the project
    let output = project.compile()?;

    if output.has_compiler_errors() {
        println!("Compiler errors:");
        for error in output.output().errors {
            println!("{}", error);
        }
        eyre::bail!("Compilation failed");
    }

    // Assuming there's only one contract in the compilation output
    let contract = output
        .find(contract_path, None)
        .ok_or_else(|| eyre::eyre!("Contract not found in compilation output"))?;

    Ok(contract.clone())
}

// Function to extract ABI, bytecode, and contract name
fn extract_abi_bytecode(
    compiled: artifacts::CompiledContract,
) -> eyre::Result<(Abi, Bytes, String)> {
    let abi = compiled.abi.unwrap();
    let bytecode = compiled.bytecode.unwrap().object.into_bytes().unwrap();
    let contract_name = compiled.name;

    Ok((abi, bytecode, contract_name))
}

// Function to verify the contract on Etherscan
async fn verify_contract(
    etherscan_api_key: String,
    contract_address: Address,
    contract_name: String,
    contract_path: &str,
    abi: Abi,
    bytecode: Bytes,
    chain_id: u64,
) -> eyre::Result<()> {
    use ethers::etherscan::Client;
    use ethers::solc::CompilerInput;

    // Read the contract source code
    let source_code = std::fs::read_to_string(contract_path)?;

    // Create an Etherscan client
    let etherscan = Client::new(chain_id.into(), etherscan_api_key)?;

    // Prepare verification metadata
    let metadata = ethers::etherscan::contract::SourceCodeMetadata {
        contract_name: format!("{}:{}", contract_path, contract_name),
        source_code: ethers::etherscan::contract::SourceCode::SingleFile(source_code),
        compiler_version: format!("v{}", ethers::solc::Solc::default_version().await?),
        optimization_used: true,
        runs: Some(200),
        constructor_arguments: None, // Provide if your contract has constructor arguments
        // Additional fields can be set as needed
        ..Default::default()
    };

    // Submit contract verification
    let resp = etherscan
        .submit_contract_verification(&contract_address, metadata)
        .await?;

    println!("Verification submitted. GUID: {}", resp.guid);

    // Wait for verification to process
    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;

    // Check verification status
    let status = etherscan
        .check_contract_verification_status(&resp.guid)
        .await?;

    println!("Verification status: {}", status);

    Ok(())
}
