use rand::rngs::OsRng;
use secp256k1::Secp256k1;
use secp256k1::hashes::{sha256::Hash as Sha256Hash, Hash};

pub const DEFAULT_VERSION_BYTE: u8 = 0x80;

pub fn bytes_32_to_wif(mut bytes: Vec<u8>, compressed: bool, version_byte: u8) -> String {
    bytes.insert(0, version_byte);
    if compressed {
        bytes.push(0x01);  // compressed
    }
    let checksum = &Sha256Hash::hash(
        &Sha256Hash::hash(&bytes)
            .to_byte_array()
    )[..4];
    bytes.extend(checksum);
    let wif = bs58::encode(bytes).into_string();
    return wif;
}

pub fn new_secret_key_32bytes() -> Vec<u8> {
    let secp = Secp256k1::new();
    let (secret_key, _) = secp.generate_keypair(&mut OsRng);
    let secret_key_bytes = secret_key.secret_bytes().to_vec();
    return secret_key_bytes;
}

pub fn new_secret_key_wif(compressed: bool, version_byte: u8) -> String {
    let secret_key_bytes = new_secret_key_32bytes();
    let wif = bytes_32_to_wif(secret_key_bytes, compressed, version_byte);
    return wif;
}

pub fn new_secret_key_wif_default_version(compressed: bool) -> String {
    return new_secret_key_wif(compressed, DEFAULT_VERSION_BYTE);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_string_to_u8_vector(hex_string: &str) -> Option<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut digits = hex_string.chars();
    
        while let (Some(upper), Some(lower)) = (digits.next(), digits.next()) {
            let byte = match u8::from_str_radix(&format!("{}{}", upper, lower), 16) {
                Ok(value) => value,
                Err(_) => return None,
            };
            bytes.push(byte);
        }
    
        Some(bytes)
    }

    #[test]
    // ref: https://iancoleman.io/bitcoin-key-compression/
    fn test_bytes_to_wif() {
        let bytes = hex_string_to_u8_vector("3f4c53c9a69fc70b008067b842c946e444f035aef4ffb8878e044084506f893e").unwrap();
        let compressed_key = bytes_32_to_wif(bytes.to_owned(), true, DEFAULT_VERSION_BYTE);
        assert_eq!(compressed_key, "KyLkhT5K4zMGCFErLttxLS5GNNtyGE92JR1fYcX5qk5Q8aoRkyrd");
        let uncompressed_key = bytes_32_to_wif(bytes, false, DEFAULT_VERSION_BYTE);
        assert_eq!(uncompressed_key, "5JJASvwSqwECbik1kvu6jvG1mGPCPUu7e2Mzft6TUMdo2NW4ZLy");
    }
}