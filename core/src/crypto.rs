use hex;
use md5::{Digest, Md5};

pub fn md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let r = hasher.finalize();
    return hex::encode(r);
}
