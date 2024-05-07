use rand::rngs::OsRng;
use secp256k1::Secp256k1;
use secp256k1::hashes::{sha256::Hash as Sha256Hash, ripemd160::Hash as Ripemp160Hash, Hash};

pub struct BitcoinKeypair {
    pub secret_key_wif: String,
    pub public_key: String,
    pub address: String,
}

impl BitcoinKeypair {
    pub fn new() -> BitcoinKeypair {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let pk_sha256 = Sha256Hash::hash(&public_key.serialize()).to_byte_array();
        let mut pk_ripemp160 = Ripemp160Hash::hash(&pk_sha256).to_byte_array().to_vec();
        pk_ripemp160.insert(0, 0x00);
        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&pk_ripemp160)
                .to_byte_array()
        )[..4];
        pk_ripemp160.extend(checksum);
        let address = bs58::encode(pk_ripemp160).into_string();

        let secret_key_bytes = secret_key.secret_bytes().to_vec();
        secret_key_bytes.insert(0, 0x80);

        let checksum = &Sha256Hash::hash(
            &Sha256Hash::hash(&secret_key_bytes)
                .to_byte_array()
        )[..4];
        secret_key_bytes.extend(checksum);

        let wif_secret_key = bs58::encode(secret_key_bytes).into_string();
    }
}