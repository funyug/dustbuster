use std::collections::HashMap;
use std::error::Error;
use bitcoin::{Amount, FeeRate, OutPoint, Psbt, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};
use bitcoin::transaction::Version;
use bitcoind::bitcoincore_rpc::bitcoincore_rpc_json::ListUnspentResultEntry;

/// Filters and returns dust UTXOs from a given list of UTXOs.
///
/// Dust UTXOs are defined as those whose amount is lower than the minimal non-dust amount
/// calculated using the provided `min_relay_fee`.
///
/// # Arguments
///
/// * `utxos` - A reference to a vector of `ListUnspentResultEntry` representing available UTXOs.
/// * `min_relay_fee` - The minimum relay fee rate (in sat/vB) used to determine dust.
/// * `address` - An optional address to filter UTXOs by. If `Some(address)`, only UTXOs belonging to that address are considered.
///
/// # Returns
///
/// Returns a `Result` containing a vector of dust UTXOs or an error.
///
/// # Errors
///
/// * If an issue occurs while filtering UTXOs.
/// * If an address is specified but no UTXOs match that address.
///
pub fn get_dust_utxos(utxos: &Vec<ListUnspentResultEntry>, min_relay_fee: u64, address: &Option<String>) -> Result<Vec<ListUnspentResultEntry>, Box<dyn Error>> {
    let mut filtered_utxos = utxos;
    #[allow(unused_assignments)]
    let mut utxos_by_address = HashMap::new();
    if !address.is_none() {
        let filter_address: &String = address.as_ref().unwrap();
        utxos_by_address = get_utxos_by_address(utxos);
        filtered_utxos = utxos_by_address.get(filter_address).unwrap();
    }
    let dust_utxos: Vec<_> = filtered_utxos.iter()
        .filter(|utxo| {
            let min_amount = utxo.script_pub_key.minimal_non_dust_custom(FeeRate::from_sat_per_vb_unchecked(min_relay_fee));
            min_amount > utxo.amount
        })
        .cloned()
        .collect();
    Ok(dust_utxos)
}

/// Creates a minimal OP_RETURN output script.
///
/// OP_RETURN outputs are provably unspendable and are commonly used for embedding data into transactions.
///
/// # Returns
///
/// Returns a byte vector representing the OP_RETURN script.
///
pub fn create_op_return_output_script() -> Vec<u8> {
    let mut script_pubkey = Vec::new();
    script_pubkey.push(0x6a); // OP_RETURN opcode
    script_pubkey
}

/// Creates a PSBT (Partially Signed Bitcoin Transaction) that spends dust UTXOs to an OP_RETURN output.
///
/// This function takes a list of UTXOs and constructs an unsigned transaction that spends them
/// to an OP_RETURN output, effectively removing them from circulation.
///
/// # Arguments
///
/// * `utxos` - A reference to a vector of UTXOs to be included in the transaction.
/// * `utxo_count` - The number of UTXOs to include in the transaction.
///
/// # Returns
///
/// Returns a `Result` containing the created PSBT or an error.
///
/// # Errors
///
/// * If the PSBT creation fails due to an issue with transaction inputs or outputs.
///
pub fn create_dust_psbt(utxos: &Vec<ListUnspentResultEntry>, utxo_count: u64) -> Result<Psbt, bitcoin::psbt::Error> {
    let inputs: Vec<TxIn> = utxos.iter().take(utxo_count as usize).map(|utxo| {
        TxIn {
            previous_output: OutPoint { txid: utxo.txid, vout: utxo.vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0xFFFFFFFD),
            witness: Witness::new(),
        }
    }).collect();

    let tx_out = TxOut {
        value: Amount::from_sat(0),
        script_pubkey: ScriptBuf::from_bytes(create_op_return_output_script()),
    };

    let outputs: Vec<TxOut> = vec![tx_out];

    let unsigned_tx = Transaction {
        version: Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: inputs,
        output: outputs,
    };

    Psbt::from_unsigned_tx(unsigned_tx)
}

/// Groups UTXOs by their associated Bitcoin address.
///
/// This function takes a list of UTXOs and organizes them into a `HashMap`
/// where the keys are Bitcoin addresses and the values are vectors of UTXOs belonging to each address.
///
/// # Arguments
///
/// * `utxos` - A reference to a vector of UTXOs to be grouped.
///
/// # Returns
///
/// Returns a `HashMap<String, Vec<ListUnspentResultEntry>>` mapping addresses to their respective UTXOs.
///
pub fn get_utxos_by_address(utxos: &Vec<ListUnspentResultEntry>) -> HashMap<String, Vec<ListUnspentResultEntry>> {
    let mut addresses: HashMap<String, Vec<ListUnspentResultEntry>> = HashMap::new();

    for utxo in utxos {
        if let Some(address) = &utxo.address {
            addresses.entry(address.clone().assume_checked().to_string()).or_default().push(utxo.clone());
        }
    }

    addresses
}