#[cfg(not(target_arch = "wasm32"))]
use halo2curves::bn256::{Fr, G1Affine};
#[cfg(not(target_arch = "wasm32"))]
use log::debug;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use snark_verifier::loader::evm::deploy_and_call;
#[cfg(not(target_arch = "wasm32"))]
use snark_verifier::loader::evm::encode_calldata;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use super::Snark;

/// Aggregate proof generation for EVM
pub mod aggregation;
/// Simple (single) proof generation for EVM
pub mod single;

#[derive(Error, Debug)]
/// Errors related to evm verification
pub enum EvmVerificationError {
    /// If the Solidity verifier worked but returned false
    #[error("Solidity verifier found the proof invalid")]
    InvalidProof,
    /// If the Solidity verifier threw and error (e.g. OutOfGas)
    #[error("Execution of Solidity code failed")]
    SolidityExecution,
    /// EVM execution errors
    #[error("EVM execution of raw code failed")]
    RawExecution,
    /// EVM verify errors
    #[error("evm verification reverted")]
    Reverted,
    /// EVM verify errors
    #[error("evm deployment failed")]
    Deploy,
}
/// YulCode type which is just an alias of string
pub type YulCode = String;

/// Defines the proof generated by a model / circuit suitably for serialization/deserialization.  
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeploymentCode {
    pub(crate) code: Vec<u8>,
}
impl DeploymentCode {
    /// Return len byte code
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// If no byte code
    pub fn is_empty(&self) -> bool {
        self.code.len() == 0
    }
    /// Return (inner) byte code
    pub fn code(&self) -> &Vec<u8> {
        &self.code
    }
    /// Saves the DeploymentCode to a specified `path`.
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string(&self).map_err(Box::<dyn Error>::from)?;

        let mut file = std::fs::File::create(path).map_err(Box::<dyn Error>::from)?;
        file.write_all(serialized.as_bytes())
            .map_err(Box::<dyn Error>::from)
    }

    /// Load a json serialized proof from the provided path.
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path).map_err(Box::<dyn Error>::from)?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(Box::<dyn Error>::from)?;
        serde_json::from_str(&data).map_err(Box::<dyn Error>::from)
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Verify by executing bytecode with instance variables and proof as input
pub fn evm_verify(
    deployment_code: DeploymentCode,
    snark: Snark<Fr, G1Affine>,
) -> Result<(), Box<dyn Error>> {
    use log::error;

    debug!("evm deployment code length: {:?}", deployment_code.len());

    let calldata = encode_calldata(&snark.instances, &snark.proof);
    match deploy_and_call(deployment_code.code, calldata) {
        Ok(gas) => debug!("gas used for call: {}", gas),
        Err(e) => {
            error!("evm deployment failed: {:?}", e);
            return Err(Box::new(EvmVerificationError::Deploy));
        }
    }
    Ok(())
}
