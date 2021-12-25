use crypto::digest::Digest;
use ring::digest::{Context, SHA256};
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use std::iter::repeat;
use std::time::{SystemTime, UNIX_EPOCH};

/// 获取当前时间戳，单位：ms
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

/// 计算 sha256 哈希值
pub fn sha256_digest(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    digest.as_ref().to_vec()
}

/// 计算 ripemd160 哈希值
pub fn ripemd160_digest(data: &[u8]) -> Vec<u8> {
    let mut ripemd160 = crypto::ripemd160::Ripemd160::new();
    ripemd160.input(data);
    let mut buf: Vec<u8> = repeat(0).take(ripemd160.output_bytes()).collect();
    ripemd160.result(&mut buf);
    return buf;
}

/// base58 编码
pub fn base58_encode(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// base58 解码
pub fn base58_decode(data: &[u8]) -> Vec<u8> {
    bs58::decode(data).into_vec().unwrap()
}

/// 创建密钥对（椭圆曲线加密）
pub fn new_key_pair() -> Vec<u8> {
    let rng = SystemRandom::new();
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
    let key_pair =
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).unwrap();
    key_pair.public_key().as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use data_encoding::HEXLOWER;
    use ring::rand::SystemRandom;
    use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

    #[test]
    fn test_sha256_digest() {
        // sha256 会产生256位的哈希值，作为消息的摘要。这个摘要相当于一个32个字节的数组，通常有一个长度为64的16进制
        // 字符串表示，其中一个字节等于8位，一个16进制的字符长度为4位。
        let digest = crate::sha256_digest("hello".as_bytes());
        // 16进制编码输出
        let hex_digest = HEXLOWER.encode(digest.as_slice());
        println!("SHA-256 digest is {}", hex_digest)
    }

    #[test]
    fn test_ripemd160() {
        let bytes = crate::ripemd160_digest("mars".as_bytes());
        let hex_str = HEXLOWER.encode(bytes.as_slice());
        // dd2324928f0552d4f4c6e57d9e5f6009ab085d85
        println!("ripemd160 digest is {}", hex_str)
    }

    #[test]
    fn test_base58() {
        let sign = "dd2324928f0552d4f4c6e57d9e5f6009ab085d85";
        let base58_sign = crate::base58_encode(sign.as_bytes());

        let decode_bytes = crate::base58_decode(base58_sign.as_bytes());
        let decode_str = String::from_utf8(decode_bytes).unwrap();
        assert_eq!(sign, decode_str.as_str());
    }

    #[test]
    fn test_ecdsa() {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).unwrap();
        for b in key_pair.public_key().as_ref() {
            print!("{:02x}", *b);
        }
    }
}
