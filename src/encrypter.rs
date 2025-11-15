

pub struct Encrypter;
impl Encrypter {
    pub fn encrypt_xor_str(input: &str, key: &[u8]) -> String {
        let encrypted: Vec<u8> = input
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        
        return base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypted);
    }
}