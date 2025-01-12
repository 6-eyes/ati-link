use sha256::digest;

pub trait Checksum {
    fn generate(&self, bytes: &[u8]) -> String;
    fn valdate(&self, bytes: &[u8], hash: &str) -> bool {
        self.generate(bytes) == hash
    }
}

pub struct Sha256;

impl Checksum for Sha256 {
    fn generate(&self, bytes: &[u8]) -> String {
        digest(bytes)
    }
}

pub struct Md5;

impl Checksum for Md5 {
    fn generate(&self, bytes: &[u8]) -> String {
        let digest = md5::compute(bytes);
        format!("{digest:x}")
    }
}

#[test]
fn test_sha256() {
    let marker = super::generate_eof_marker();
    let checksum = Sha256.generate(&marker);
    assert!(Sha256.valdate(&marker, &checksum))
}

#[test]
fn test_md5() {
    let marker = super::generate_eof_marker();
    let checksum = Md5.generate(&marker);
    assert!(Md5.valdate(&marker, &checksum))
}
