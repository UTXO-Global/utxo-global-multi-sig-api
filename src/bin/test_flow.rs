use ckb_sdk::{
    transaction::{
        builder::{CkbTransactionBuilder, SimpleTransactionBuilder},
        handler::HandlerContexts,
        input::InputIterator,
        signer::{SignContexts, TransactionSigner},
        TransactionBuilderConfiguration,
    },
    unlock::MultisigConfig,
    Address, NetworkInfo, TransactionWithScriptGroups,
};
use ckb_types::{
    bytes::Bytes, core::Capacity, h160, h256, packed::Transaction, prelude::IntoTransactionView,
};
use std::{error::Error as StdErr, str::FromStr};
use utxo_global_multi_sig_api::{
    repositories::ckb::{add_signature_to_witness, get_multisig_config},
    serialize::multi_sig_account::SignerInfo,
};

fn get_tx_group_with_script(multisig_config: &MultisigConfig) -> TransactionWithScriptGroups {
    let network_info = NetworkInfo::testnet();

    let configuration =
        TransactionBuilderConfiguration::new_with_network(network_info.clone()).unwrap();

    // ckt1qpw9q60tppt7l3j7r09qcp7lxnp3vcanvgha8pmvsa3jplykxn32sqdunqvd3g2felqv6qer8pkydws8jg9qxlca0st5v
    let sender = multisig_config.to_address(network_info.network_type, None);

    let receiver = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2qf8keemy2p5uu0g0gn8cd4ju23s5269qk8rg4r").unwrap();

    // Query to RPC to get the available cells
    let iterator = InputIterator::new_with_address(&[sender], &network_info);
    let mut builder = SimpleTransactionBuilder::new(configuration, iterator);

    // Define outputs - based on outputs, the sdk will auto select usable inputs
    builder.add_output(&receiver, Capacity::shannons(510_0000_0000u64));

    builder
        .build(&HandlerContexts::new_multisig(multisig_config.clone()))
        .unwrap()
}

fn main() -> Result<(), Box<dyn StdErr>> {
    let network_info = NetworkInfo::testnet();

    // ------  1. Create new multi-sig account from multiple account ------
    let multisig_config = MultisigConfig::new_with(
        vec![
            // ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqd20cjzl05a0w0wj99lszmdzfndaqdcruq4lap8z
            h160!("0xaa7e242fbe9d7b9ee914bf80b6d1266de81b81f0"),
            // ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqf0hg6dacn9q2qtgv2t74sdfc7t9ke3z9spvtdsf
            h160!("0x2fba34dee2650280b4314bf560d4e3cb2db31116"),
            // ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq0em8544fda3klhfynw89d7ctjmqhenmlqq949sq
            h160!("0xf9d9e95aa5bd8dbf74926e395bec2e5b05f33dfc"),
        ],
        0,
        2,
    )?;
    // ckt1qpw9q60tppt7l3j7r09qcp7lxnp3vcanvgha8pmvsa3jplykxn32sqdunqvd3g2felqv6qer8pkydws8jg9qxlca0st5v
    let sender = multisig_config.to_address(network_info.network_type, None);

    // Get multi-sig config
    let (multi_sig_address, multi_sig_witness_data) = get_multisig_config(vec![
        SignerInfo{
            address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqd20cjzl05a0w0wj99lszmdzfndaqdcruq4lap8z".to_string(),
            name: "Test 1".to_owned()
        },
        SignerInfo{
            address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqf0hg6dacn9q2qtgv2t74sdfc7t9ke3z9spvtdsf".to_string(),
            name: "Test 1".to_owned()
        },
        SignerInfo{
            address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq0em8544fda3klhfynw89d7ctjmqhenmlqq949sq".to_string(),
            name: "Test 1".to_owned()
        }
        ,
    ], 2).unwrap();

    let multisig_witness_bytes_len = multi_sig_witness_data.len();

    assert_eq!(sender, multi_sig_address);
    assert_eq!(
        hex::encode(multisig_config.to_witness_data()),
        multi_sig_witness_data
    );

    // ------ 2. Build new simple transfer ------
    let mut tx_with_groups = get_tx_group_with_script(&multisig_config);

    // ------ 3. Collect signatures into tx_group ------

    // signer 1
    let private_key1 = h256!("0x0837342ef863227453f4b6f371a2c544fd2becb76c0b2994e4b0bcf00243e86f");
    let signer1: TransactionSigner = TransactionSigner::new(&network_info);
    signer1.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key1, multisig_config.clone())?,
    )?;

    // signer 2
    let signer2 = TransactionSigner::new(&network_info);
    let private_key2 = h256!("0xe9698bbc8b09b2032266fe637c5aa4c5419269fba5cc7ed83cb304b0e8689eef");
    signer2.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key2, multisig_config.clone())?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    let witness = hex::encode(
        json_tx
            .inner
            .witnesses
            .first()
            .unwrap()
            .clone()
            .into_bytes(),
    );
    let witness_full = witness;
    let tx_hash = json_tx.hash;
    println!("witness full sign by lib {}", witness_full);

    // ------ 3.2 Collect signatures into seperate tx_group, to collect signature seperately ------

    let mut tx_with_groups = get_tx_group_with_script(&multisig_config);
    // signer 2
    let signer2 = TransactionSigner::new(&network_info);
    let private_key2 = h256!("0xe9698bbc8b09b2032266fe637c5aa4c5419269fba5cc7ed83cb304b0e8689eef");
    signer2.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key2, multisig_config.clone())?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    let witness = hex::encode(
        json_tx
            .inner
            .witnesses
            .first()
            .unwrap()
            .clone()
            .into_bytes(),
    );
    println!("witness2: {}", &witness);
    println!(
        "sig2: {}",
        &witness[(multi_sig_witness_data.len() + 40)..(128 + (multisig_witness_bytes_len + 40))]
    );

    let mut sig2 = vec![0; 65];
    sig2.clone_from_slice(
        hex::decode(
            &witness[(multisig_witness_bytes_len + 40)..(130 + (multisig_witness_bytes_len + 40))],
        )
        .unwrap()
        .as_ref(),
    );
    let sig2 = Bytes::from(sig2);

    let mut tx_with_groups = get_tx_group_with_script(&multisig_config);
    // signer 1
    let private_key1 = h256!("0x0837342ef863227453f4b6f371a2c544fd2becb76c0b2994e4b0bcf00243e86f");
    let signer1 = TransactionSigner::new(&network_info);

    signer1.sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_multisig_h256(&private_key1, multisig_config.clone())?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());

    let witness = hex::encode(
        json_tx
            .inner
            .witnesses
            .first()
            .unwrap()
            .clone()
            .into_bytes(),
    );

    println!("witness1: {}", &witness);
    println!(
        "sig1: {}",
        &witness[(multisig_witness_bytes_len + 40)..(128 + (multisig_witness_bytes_len + 40))]
    );

    let mut sig1 = vec![0; 65];
    sig1.clone_from_slice(
        hex::decode(
            &witness[(multisig_witness_bytes_len + 40)..(130 + (multisig_witness_bytes_len + 40))],
        )
        .unwrap()
        .as_ref(),
    );
    let sig1 = Bytes::from(sig1);

    let signatures = vec![sig2, sig1];
    println!("signatures {:?}", signatures);
    let tx = Transaction::from(json_tx.clone().inner).into_view();
    let tx = add_signature_to_witness(2, &tx, &multi_sig_witness_data, signatures).unwrap();
    let json_tx_2 = ckb_jsonrpc_types::TransactionView::from(tx);

    // ----------- 4. Review Tx -----------
    let witness = hex::encode(
        json_tx_2
            .inner
            .witnesses
            .first()
            .unwrap()
            .clone()
            .into_bytes(),
    );

    println!("witness {}", witness);

    assert_eq!(witness_full, witness);
    assert_eq!(tx_hash, json_tx_2.hash);

    println!("tx: {}", serde_json::to_string_pretty(&json_tx_2).unwrap());
    // send_transaction(tx, None);
    Ok(())
}
