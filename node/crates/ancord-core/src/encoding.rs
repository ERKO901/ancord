//! Kódování podle `spec/PROTOCOL.md` §1.1: b64u bez paddingu (striktní) a base32 malými písmeny.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

pub fn b64u_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Striktní dekodér: bez paddingu, jen abeceda base64url, kanonická podoba.
pub fn b64u_decode(s: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(s).ok()
}

pub fn b64u_decode_array<const N: usize>(s: &str) -> Option<[u8; N]> {
    b64u_decode(s)?.try_into().ok()
}

/// RFC 4648 base32, malá písmena, bez paddingu.
pub fn base32_encode(bytes: &[u8]) -> String {
    data_encoding::BASE32_NOPAD.encode(bytes).to_ascii_lowercase()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    data_encoding::HEXLOWER.encode(bytes)
}

pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    data_encoding::HEXLOWER_PERMISSIVE.decode(s.as_bytes()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64u_roundtrip_and_strictness() {
        assert_eq!(b64u_encode(&[0xfb, 0xff]), "-_8");
        assert_eq!(b64u_decode("-_8"), Some(vec![0xfb, 0xff]));
        // padding a nekanonické koncové bity se odmítají
        assert_eq!(b64u_decode("-_8="), None);
        assert_eq!(b64u_decode("-_9"), None);
        assert_eq!(b64u_decode("a+b"), None);
    }

    #[test]
    fn base32_lowercase() {
        assert_eq!(base32_encode(b"foobar"), "mzxw6ytboi");
    }
}
