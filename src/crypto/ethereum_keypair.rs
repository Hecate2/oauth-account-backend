use std::error::Error;
use rand::rngs::OsRng;
use secp256k1;
use secp256k1::Secp256k1;
use secp256k1::hashes::{sha256::Hash as Sha256Hash, ripemd160::Hash as Ripemp160Hash, Hash};
use sha3::{Digest, Keccak256};
use crate::crypto::secret_key::bytes_32_to_wif;

const ADDRESS_VERSION_BYTE_MAINNET: u8 = 0x00;
const WIF_VERSION_BYTE_MAINNET: u8 = 0x80;

pub struct EthereumKeypair {
    pub secret_key_compressed_wif: String,
    pub public_key: String,
    pub address: String,
}

impl EthereumKeypair {
    pub fn new() -> EthereumKeypair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        return EthereumKeypair::from_keypair(secret_key, public_key);
    }

    // checksum not checked!
    pub fn from_compressed_wif(wif: String) -> Result<EthereumKeypair, Box<dyn Error>> {
        let mut secret_key_bytes: [u8; 38] = [0x00; 38];
        bs58::decode(wif).onto(&mut secret_key_bytes)?;
        Ok(EthereumKeypair::from_secret_key_slice(&secret_key_bytes[1..33])?)
    }

    pub fn from_secret_key_slice(s: &[u8]) -> Result<EthereumKeypair, Box<dyn Error>> {
        let secp = Secp256k1::new();
        let sk = secp256k1::SecretKey::from_slice(s)?;
        let keypair = secp256k1::Keypair::from_secret_key(&secp, &sk);
        let (secret_key, public_key) = (keypair.secret_key(), keypair.public_key());
        Ok(EthereumKeypair::from_keypair(secret_key, public_key))
    }

    pub fn from_keypair(secret_key: secp256k1::SecretKey, public_key: secp256k1::PublicKey) -> EthereumKeypair {
        let public_key_bytes = public_key.serialize();
        let mut hasher = Keccak256::new();
        hasher.update(&public_key.serialize_uncompressed()[1..]);
        let public_key_hash = hasher.finalize();
        let address = format!("{}{}", "0x", hex::encode(&public_key_hash[12..]));

        let public_key_string: String = public_key_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("");

        let secret_key_bytes = secret_key.secret_bytes().to_vec();
        let secret_key_compressed_wif = bytes_32_to_wif(secret_key_bytes, true, WIF_VERSION_BYTE_MAINNET);        

        EthereumKeypair{ secret_key_compressed_wif: secret_key_compressed_wif, public_key: public_key_string, address: address}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addr() {
        let keypair = EthereumKeypair::from_compressed_wif("KyLkhT5K4zMGCFErLttxLS5GNNtyGE92JR1fYcX5qk5Q8aoRkyrd");
        assert_eq!(keypair.address, "0x5cf4e71E0d8466A958934Ce4e0D00b8ed1A3A973");
    }
}