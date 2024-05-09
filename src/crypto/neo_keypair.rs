use std::error::Error;
use rand::rngs::OsRng;
use p256::ecdsa::{SigningKey, VerifyingKey};
use secp256k1::hashes::{sha256::Hash as Sha256Hash, ripemd160::Hash as Ripemp160Hash, Hash};
use crate::crypto::secret_key::bytes_32_to_wif;

const ADDRESS_VERSION_BYTE_MAINNET: u8 = 53;
const WIF_VERSION_BYTE_MAINNET: u8 = 0x80;

pub struct NeoKeypair {
    pub secret_key_compressed_wif: String,
    pub public_key: String,
    pub address: String,
}

impl NeoKeypair {
    pub fn new() -> NeoKeypair {
        let secret_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&secret_key);
        return NeoKeypair::from_keypair(secret_key, public_key);
    }

    // checksum not checked!
    pub fn from_compressed_wif(wif: &str) -> Result<NeoKeypair, Box<dyn Error>> {
        let mut secret_key_bytes: [u8; 38] = [0x00; 38];
        bs58::decode(wif).onto(&mut secret_key_bytes)?;
        Ok(NeoKeypair::from_secret_key_slice(&secret_key_bytes[1..33])?)
    }

    pub fn from_secret_key_slice(s: &[u8]) -> Result<NeoKeypair, Box<dyn Error>> {
        let secret_key = SigningKey::from_slice(s).unwrap();
        let public_key = VerifyingKey::from(&secret_key);
        Ok(NeoKeypair::from_keypair(secret_key, public_key))
    }

    pub fn from_keypair(secret_key: p256::ecdsa::SigningKey, public_key: p256::ecdsa::VerifyingKey) -> NeoKeypair {
        let public_key_bytes = public_key.to_encoded_point(true).to_bytes();
        let mut verification_script: Vec<u8> = Vec::new();
        verification_script.extend_from_slice(&[0x0c, 0x21]);  // PUSHDATA1
        verification_script.extend_from_slice(&public_key_bytes);
        verification_script.extend_from_slice(&[0x41, 0x56, 0xe7, 0xb3, 0x27]);  // SYSCALL System.Crypto.CheckSig
        let script_sha256 = Sha256Hash::hash(&verification_script).to_byte_array();
        let mut script_hash = Ripemp160Hash::hash(&script_sha256).to_byte_array().to_vec();
        script_hash.insert(0, ADDRESS_VERSION_BYTE_MAINNET);
        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&script_hash)
                .to_byte_array()
        )[..4];
        script_hash.extend(checksum);
        let address = bs58::encode(script_hash).into_string();

        let public_key_string: String = public_key_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("");

        let secret_key_bytes = secret_key.to_bytes().to_vec();
        let secret_key_compressed_wif = bytes_32_to_wif(secret_key_bytes, true, WIF_VERSION_BYTE_MAINNET);        

        NeoKeypair{ secret_key_compressed_wif: secret_key_compressed_wif, public_key: public_key_string, address: address}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // ref: https://neo.org/converter/index
    fn test_addr() {
        let keypair = NeoKeypair::from_compressed_wif("KyLkhT5K4zMGCFErLttxLS5GNNtyGE92JR1fYcX5qk5Q8aoRkyrd").unwrap();
        assert_eq!(keypair.address, "NUrR2m4hRyYTdzFseDZzUKLhj5YLJtjEN2");
    }
}