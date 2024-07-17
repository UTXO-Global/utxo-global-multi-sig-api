use ckb_sdk::{constants::MULTISIG_TYPE_HASH, Address, AddressPayload, CodeHashIndex, NetworkType};
use ckb_types::{bytes::Bytes, core::ScriptHashType, h160, prelude::Pack};

fn main() {
    // https://explorer.nervos.org/en/transaction/0x72a99e1fccd7ce2194cd703266ad29f337c74e0cd49400c8ab63c89c5d3ff73e
    // New:        ckb1qpw9q60tppt7l3j7r09qcp7lxnp3vcanvgha8pmvsa3jplykxn32sqwdvtqqky3n3ntlmdqmv8ectytr6sr90vs9a9rmn
    // Deprecated: ckb1qyq9hss45s4yc2crmdk06fwwcuewe3dm4hhqr4ws7y

    // Full
    let arg = h160!("0xcd62c00b12338cd7fdb41b61f3859163d40657b2");

    let payload = AddressPayload::new_full(
        ScriptHashType::Type,
        MULTISIG_TYPE_HASH.pack(),
        Bytes::copy_from_slice(arg.as_bytes()),
    );
    let address = Address::new(NetworkType::Testnet, payload, true);
    println!("address full {}", address);

    // Short
    let payload = AddressPayload::new_short(CodeHashIndex::Multisig, arg);
    let address = Address::new(NetworkType::Testnet, payload, true);
    println!("address short {}", address);
}
