extern crate bitcoin;
extern crate ledger_bitcoin;

use bitcoin::network::constants::Network;
use ledger_bitcoin::Ledger;

fn main() {
    let ledger = Ledger::new(Network::Bitcoin)
        .expect("can't find ledger");

    println!("{:?}", ledger.get_firmware_version().unwrap());
}