extern crate bitcoin;
extern crate ledger_bitcoin;

use ledger_bitcoin::Ledger;
use bitcoin::network::constants::Network;
use bitcoin::util::address::Address;
use bitcoin::util::bip32::{ChildNumber, DerivationPath};

fn main() {
    let ledger = Ledger::new(Network::Bitcoin)
        .expect("can't find ledger");

    let account_1_receive_key = ledger.get_key(DerivationPath::from(&[
        ChildNumber::from_hardened_idx(49).unwrap(),
        ChildNumber::from_hardened_idx(0).unwrap(),
        ChildNumber::from_hardened_idx(0).unwrap(),
        ChildNumber::from_normal_idx(0).unwrap(),
    ][..])).expect("can't fetch pub key");

    let account_1_change_key = ledger.get_key(DerivationPath::from(&[
        ChildNumber::from_hardened_idx(49).unwrap(),
        ChildNumber::from_hardened_idx(0).unwrap(),
        ChildNumber::from_hardened_idx(0).unwrap(),
        ChildNumber::from_normal_idx(1).unwrap(),
    ][..])).expect("can't fetch pub key");

    let secp_ctx = ledger_bitcoin::bitcoin::secp256k1::Secp256k1::new();

    println!("Receive keys                          Change keys");
    for child_number in 0..16 {
        let rcv_key = account_1_receive_key.ckd_pub(&secp_ctx, ChildNumber::from(child_number))
            .expect("key derivation failed");
        let chg_key = account_1_change_key.ckd_pub(&secp_ctx, ChildNumber::from(child_number))
            .expect("key derivation failed");

        let rcv_addr = Address::p2shwpkh(&rcv_key.public_key, Network::Bitcoin);
        let chg_addr = Address::p2shwpkh(&chg_key.public_key, Network::Bitcoin);

        println!("{}    {}", rcv_addr, chg_addr);
    }
}
