use ckb_sdk::{
    transaction::{
        builder::{CkbTransactionBuilder, SimpleTransactionBuilder},
        handler::HandlerContexts,
        input::InputIterator,
        signer::{SignContexts, TransactionSigner},
        TransactionBuilderConfiguration,
    },
    unlock::MultisigConfig,
    Address, NetworkInfo,
};
use ckb_types::{core::Capacity, h160, h256};
use std::{error::Error as StdErr, str::FromStr};

fn main() -> Result<(), Box<dyn StdErr>> {
    let network_info = NetworkInfo::testnet();
    // let network_info = NetworkInfo::new(NetworkType::Testnet, "http://localhost:8114".to_string());

    let configuration = TransactionBuilderConfiguration::new_with_network(network_info.clone())?;

    let multisig_config = MultisigConfig::new_with(
        vec![
            h160!("0x7336b0ba900684cb3cb00f0d46d4f64c0994a562"),
            h160!("0x5724c1e3925a5206944d753a6f3edaedf977d77f"),
        ],
        0,
        2,
    )?;
    let sender = multisig_config.to_address(network_info.network_type, None);
    let receiver = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2qf8keemy2p5uu0g0gn8cd4ju23s5269qk8rg4r")?;
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
    let private_key2 = h256!("0x7438f7b35c355e3d2fb9305167a31a72d22ddeafb80a21cc99ff6329d92e8087");
    signer2.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key2, multisig_config.clone())?,
    )?;
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    let private_key1 = h256!("0x4fd809631a6aa6e3bb378dd65eae5d71df895a82c91a615a1e8264741515c79c");
    let signer1 = TransactionSigner::new(&network_info);
    signer1.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key1, multisig_config.clone())?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    Ok(())
}
