use rand::rngs::OsRng;
use secp256k1::Secp256k1;
use secp256k1::hashes::{sha256::Hash as Sha256Hash, ripemd160::Hash as Ripemp160Hash, Hash};

const ADDRESS_VERSION_BYTE_MAINNET: u8 = 0x00;
const WIF_VERSION_BYTE_MAINNET: u8 = 0x80;

pub struct BitcoinKeypair {
    pub secret_key_wif: String,
    pub public_key: String,
    pub address: String,
}

impl BitcoinKeypair {
    pub fn new() -> BitcoinKeypair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let public_key_bytes = public_key.serialize();
        let public_key_string: String = public_key_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("");
        let pk_sha256 = Sha256Hash::hash(&public_key_bytes).to_byte_array();
        let mut pk_ripemp160 = Ripemp160Hash::hash(&pk_sha256).to_byte_array().to_vec();
        pk_ripemp160.insert(0, ADDRESS_VERSION_BYTE_MAINNET);
        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&pk_ripemp160)
                .to_byte_array()
        )[..4];
        pk_ripemp160.extend(checksum);
        let address = bs58::encode(pk_ripemp160).into_string();

        let mut secret_key_bytes = secret_key.secret_bytes().to_vec();
        secret_key_bytes.insert(0, WIF_VERSION_BYTE_MAINNET);
        secret_key_bytes.push(0x01);  // compressed

        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&secret_key_bytes)
                .to_byte_array()
        )[..4];
        secret_key_bytes.extend(checksum);
        let secret_key_wif = bs58::encode(secret_key_bytes).into_string();
        
        BitcoinKeypair{ secret_key_wif: secret_key_wif, public_key: public_key_string, address: address}
    }
}