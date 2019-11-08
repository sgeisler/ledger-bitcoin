extern crate bitcoin;
extern crate ledger;

use bitcoin::util::key as btckey;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1;
use bitcoin::util::bip32::{ChainCode, ChildNumber, DerivationPath, ExtendedPubKey};
use ledger::ApduCommand;
use std::convert::{TryFrom, TryInto};

pub use ledger::Error as LedgerError;

pub struct Ledger {
    device_handle: ledger::LedgerApp,
    network: Network,
}

struct KeyRequestAnswer<'a> {
    pub_key: &'a [u8],
    /// Should always be 32 bytes long!
    chain_code: &'a [u8],
}

impl Ledger {
    pub fn new(network: Network) -> Result<Ledger, LedgerError> {
        Ok(Ledger {
            device_handle: ledger::LedgerApp::new()?,
            network
        })
    }

    pub fn get_key(&self, path: DerivationPath) -> Result<ExtendedPubKey, Error> {
        let path_len = path.as_ref().len();
        if path_len > 10 {
            return Err(Error::DerivationPathTooLong)
        }

        let mut request_data = Vec::<u8>::with_capacity(1 + path_len * 4);
        request_data.push(path_len as u8);
        for child in path.as_ref() {
            let raw_child_num: u32 = (*child).into();
            request_data.extend_from_slice(&raw_child_num.to_be_bytes()[..])
        }

        let command = ApduCommand {
            cla: 0xE0,
            ins: 0x40,
            p1: 0x00,
            p2: 0x00,
            length: request_data.len() as u8,
            data: request_data
        };

        let raw_answer = self.device_handle
            .exchange(command)?;
        let answer: KeyRequestAnswer = raw_answer.data.as_slice().try_into()?;

        let pub_key = btckey::PublicKey {
            compressed: true,
            key: secp256k1::key::PublicKey::from_slice(answer.pub_key)?
        };

        let chain_code = ChainCode::from(answer.chain_code);

        Ok(ExtendedPubKey {
            network: self.network,
            depth: path_len as u8,
            parent_fingerprint: Default::default(),
            child_number: path.as_ref().last().cloned().unwrap_or(ChildNumber::from(0)),
            public_key: pub_key,
            chain_code: chain_code,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for KeyRequestAnswer<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<KeyRequestAnswer<'a>, Self::Error> {
        // read pub key
        let pk_len = bytes[0] as usize;
        if bytes.len() < 1 + pk_len {
            return Err(Error::InvalidLedgerResponse);
        }
        let pub_key = &bytes[1..(1+pk_len)];

        // skip legacy base58 address returned
        let b58_len = bytes[1+pk_len] as usize;
        if bytes.len() != (1 + pk_len + 1 + b58_len + 32) {
            return Err(Error::InvalidLedgerResponse);
        }

        // read chain code
        let chain_code = &bytes[(1+pk_len+1+b58_len)..];
        assert_eq!(chain_code.len(), 32);

        Ok(KeyRequestAnswer {
            pub_key,
            chain_code
        })
    }
}

pub enum Error {
    DerivationPathTooLong,
    InvalidLedgerResponse,
    LedgerError(LedgerError),
    Secp256k1Error(secp256k1::Error),
}

impl From<LedgerError> for Error {
    fn from(e: LedgerError) -> Self {
        Error::LedgerError(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Error::Secp256k1Error(e)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
