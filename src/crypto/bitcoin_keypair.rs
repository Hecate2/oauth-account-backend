use std::error::Error;
use rand::rngs::OsRng;
use secp256k1;
use secp256k1::Secp256k1;
use secp256k1::hashes::{sha256::Hash as Sha256Hash, ripemd160::Hash as Ripemp160Hash, Hash};
use crate::crypto::secret_key::bytes_32_to_wif;

const ADDRESS_VERSION_BYTE_MAINNET: u8 = 0x00;
const WIF_VERSION_BYTE_MAINNET: u8 = 0x80;

pub struct BitcoinKeypair {
    pub secret_key_compressed_wif: String,
    pub public_key: String,
    pub address: String,
}

impl BitcoinKeypair {
    pub fn new() -> BitcoinKeypair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        return BitcoinKeypair::from_keypair(secret_key, public_key);
    }

    // checksum not checked!
    pub fn from_compressed_wif(wif: String) -> Result<BitcoinKeypair, Box<dyn Error>> {
        let mut secret_key_bytes: [u8; 38] = [0x00; 38];
        bs58::decode(wif).onto(&mut secret_key_bytes)?;
        Ok(BitcoinKeypair::from_secret_key_slice(&secret_key_bytes[1..33])?)
    }

    pub fn from_secret_key_slice(s: &[u8]) -> Result<BitcoinKeypair, Box<dyn Error>> {
        let secp = Secp256k1::new();
        let sk = secp256k1::SecretKey::from_slice(s)?;
        let keypair = secp256k1::Keypair::from_secret_key(&secp, &sk);
        let (secret_key, public_key) = (keypair.secret_key(), keypair.public_key());
        Ok(BitcoinKeypair::from_keypair(secret_key, public_key))
    }

    pub fn from_keypair(secret_key: secp256k1::SecretKey, public_key: secp256k1::PublicKey) -> BitcoinKeypair {
        let public_key_bytes = public_key.serialize();
        let pk_sha256 = Sha256Hash::hash(&public_key_bytes).to_byte_array();
        let mut pk_ripemp160 = Ripemp160Hash::hash(&pk_sha256).to_byte_array().to_vec();
        pk_ripemp160.insert(0, ADDRESS_VERSION_BYTE_MAINNET);
        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&pk_ripemp160)
                .to_byte_array()
        )[..4];
        pk_ripemp160.extend(checksum);
        let address = bs58::encode(pk_ripemp160).into_string();

        let public_key_string: String = public_key_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("");

        let mut secret_key_bytes = secret_key.secret_bytes().to_vec();
        let secret_key_compressed_wif = bytes_32_to_wif(secret_key_bytes, true, WIF_VERSION_BYTE_MAINNET);        

        BitcoinKeypair{ secret_key_compressed_wif: secret_key_compressed_wif, public_key: public_key_string, address: address}
    }
}