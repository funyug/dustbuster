use std::error::Error;
use std::io;
use std::io::Write;
use base64::Engine;
use base64::engine::general_purpose;
use bitcoind::bitcoincore_rpc::{Client, RpcApi};
use crate::transaction::{create_dust_psbt, get_dust_utxos};

mod transaction;

/// A utility for managing and consolidating dust UTXOs in a Bitcoin wallet.
///
/// `DustBuster` provides functions to list, filter, and create transactions
/// that consolidate dust UTXOs by spending them to an OP_RETURN output.
///
/// # Fields
///
/// * `client` - A `Client` instance used to interact with a Bitcoin node.
pub struct DustBuster {
    client: Client,
}
impl DustBuster {
    /// Creates a new `DustBuster` instance with the given Bitcoin RPC client.
    ///
    /// # Arguments
    ///
    /// * `client` - A Bitcoin RPC client used to interact with the wallet.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `DustBuster`.
    ///
    pub fn new(client: Client) -> DustBuster {
        Self {
            client,
        }
    }
    /// Lists all dust UTXOs in the wallet based on the given `min_relay_fee`.
    ///
    /// This function retrieves all UTXOs from the wallet and filters out those
    /// considered "dust" based on the provided minimum relay fee rate.
    /// If dust UTXOs are found, it prompts the user to decide whether to print them.
    ///
    /// # Arguments
    ///
    /// * `min_relay_fee` - The minimum relay fee rate (in sat/vB) to classify dust UTXOs.
    /// * `address` - An optional Bitcoin address to filter UTXOs by. If `None`, all UTXOs are considered.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err(Box<dyn Error>)` if an error occurs while fetching or processing UTXOs.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// * Fetching UTXOs from the Bitcoin node fails.
    /// * Filtering UTXOs encounters an issue.
    /// * Reading user input fails.
    pub fn list_dust(&self, min_relay_fee: u64, address: &Option<String>) -> Result<(), Box<dyn Error>> {
        let utxos = self.client.list_unspent(None,None,None,None,None).unwrap();
        let dust_utxos = get_dust_utxos(&utxos, min_relay_fee, address)?;
        if dust_utxos.is_empty() {
            println!("No UTXOs found");
            return Ok(());
        }
        println!("Total Dust UTXOs: {}", dust_utxos.len());

        // Ask the user if they want to print the dust UTXOs
        print!("Do you want to print the dust UTXOs? (yes/no): ");
        io::stdout().flush()?; // Ensure the prompt is displayed before input

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "yes" || input == "y" {
            for utxo in &dust_utxos {
                println!("Txid: {} Vout: {} Amount: {}", utxo.txid, utxo.vout, utxo.amount);
            }
        }
        
        Ok(())
    }

    /// Creates an unsigned PSBT containing dust inputs spent to an empty OP_RETURN output.
    ///
    /// This function selects dust UTXOs based on the given `min_relay_fee` and spends them 
    /// in a transaction with an OP_RETURN output. The generated PSBT (Partially Signed 
    /// Bitcoin Transaction) is printed in base64 format.
    ///
    /// # Arguments
    ///
    /// * `min_relay_fee` - The minimum relay fee rate (in satoshis per vByte) used to classify dust UTXOs.
    /// * `address` - The address whose dust UTXOs should be selected. If empty, all dust UTXOs are considered.
    /// * `utxo_count` - The maximum number of UTXOs to include in the PSBT.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * No dust UTXOs are found.
    /// * The PSBT creation fails.
    /// * Any client interaction results in an error.
    ///
    /// # Panics
    ///
    /// This function panics if no dust UTXOs are found.
    pub fn create_psbt(&self, min_relay_fee: u64, address: String, utxo_count: u64) -> Result<(), Box<dyn Error>> {
        let utxos = self.client.list_unspent(None,None,None,None,None).unwrap();
        let dust_utxos = get_dust_utxos(&utxos, min_relay_fee, &Some(address))?;
        if dust_utxos.is_empty() {
            panic!("No UTXOs found");
        }
        let psbt = create_dust_psbt(&dust_utxos, utxo_count)?;
        println!("{}", general_purpose::STANDARD.encode(&psbt.serialize()));
        Ok(())
    }
}