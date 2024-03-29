extern crate bitcoin;
extern crate ledger;

use bitcoin::util::key as btckey;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1;
use bitcoin::util::bip32::{ChainCode, ChildNumber, DerivationPath, ExtendedPubKey};
use ledger::ApduCommand;
use std::convert::{TryFrom, TryInto};

pub use ledger::Error as LedgerError;

/// Ledger device handle
pub struct Ledger {
    device_handle: ledger::LedgerApp,
    network: Network,
}

struct KeyRequestAnswer<'a> {
    pub_key: &'a [u8],
    /// Should always be 32 bytes long!
    chain_code: &'a [u8],
}

#[derive(Debug)]
pub struct LedgerFirmwareVersion {
    pub features: u8,
    pub architecture: u8,
    pub firmware_version: [u8; 3],
    pub loader_version: [u8; 2],
}

impl Ledger {
    /// Get handle for first recognized and connected ledger device
    pub fn new(network: Network) -> Result<Ledger, LedgerError> {
        Ok(Ledger {
            device_handle: ledger::LedgerApp::new()?,
            network
        })
    }

    pub fn get_firmware_version(&self) -> Result<LedgerFirmwareVersion, Error> {
        self.device_handle.exchange(ApduCommand {
            cla: 0xE0,
            ins: 0xC4,
            p1: 0,
            p2: 0,
            length: 0,
            data: vec![]
        })?.data.as_slice().try_into()

    }

    /// Fetch an extended public key using a derivation path
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

impl TryFrom<&[u8]> for LedgerFirmwareVersion {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 7 {
            return Err(Error::InvalidLedgerResponse);
        }

        let f = data.to_vec();

        Ok(LedgerFirmwareVersion {
            features: data[0],
            architecture: data[1],
            firmware_version: data[2..5].try_into().unwrap(),
            loader_version: data[5..7].try_into().unwrap()
        })
    }
}

#[derive(Debug)]
pub enum Error {
    /// Ledger devices only support derivation of depth <=10
    DerivationPathTooLong,
    /// The ledger sent a response we couldn't parse
    InvalidLedgerResponse,
    /// Ledger error
    LedgerError(LedgerError),
    /// Secp256k1 error
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
}
