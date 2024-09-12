use ckb_sdk::{
    transaction::{
        builder::{CkbTransactionBuilder, SimpleTransactionBuilder},
        handler::HandlerContexts,
        input::InputIterator,
        signer::SignContexts,
        TransactionBuilderConfiguration,
    },
    unlock::MultisigConfig,
    Address, NetworkInfo,
};
use ckb_types::{core::Capacity, h160, h256};
use std::{error::Error as StdErr, str::FromStr};
use utxo_global_multi_sig_api::services::overrided::{OverrideMultisigConfig, TransactionSigner};

fn main() -> Result<(), Box<dyn StdErr>> {
    let network_info = NetworkInfo::testnet();
    // let network_info = NetworkInfo::new(NetworkType::Testnet, "http://localhost:8114".to_string());

    let configuration = TransactionBuilderConfiguration::new_with_network(network_info.clone())?;

    let multisig_config = MultisigConfig::new_with(
        vec![
            h160!("0xaa7e242fbe9d7b9ee914bf80b6d1266de81b81f0"),
            h160!("0x2fba34dee2650280b4314bf560d4e3cb2db31116"),
            h160!("0xf9d9e95aa5bd8dbf74926e395bec2e5b05f33dfc"),
        ],
        0,
        2,
    )?;
    let sender = multisig_config.to_address_override(network_info.network_type, None);
    let receiver = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqgnf80gpm2klvxhq3gt848ag67s6mztldsvt4nh0")?;
    println!("arg {:?}", hex::encode(sender.payload().args()));

    // Query to RPC to get the available cells
    let iterator = InputIterator::new_with_address(&[sender], &network_info);
    let mut builder = SimpleTransactionBuilder::new(configuration, iterator);

    // Define outputs - based on outputs, the sdk will auto select usable inputs
    builder.add_output(&receiver, Capacity::shannons(510_0000_0000u64));

    let mut tx_with_groups =
        builder.build(&HandlerContexts::new_multisig(multisig_config.clone()))?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    let signer2 = TransactionSigner::new(&network_info);
    let private_key2 = h256!("0xe9698bbc8b09b2032266fe637c5aa4c5419269fba5cc7ed83cb304b0e8689eef");
    signer2.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key2, multisig_config.clone())?,
    )?;
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!(
        "tx signer 1: {}",
        serde_json::to_string_pretty(&json_tx).unwrap()
    );

    let private_key1 = h256!("0x0837342ef863227453f4b6f371a2c544fd2becb76c0b2994e4b0bcf00243e86f");
    let signer1: TransactionSigner = TransactionSigner::new(&network_info);
    signer1.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key1, multisig_config.clone())?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!(
        "tx signer 2: {}",
        serde_json::to_string_pretty(&json_tx).unwrap()
    );

    Ok(())
}
