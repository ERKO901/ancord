//! Obsah zprávy před šifrováním ratchetem (spec §15.14): text a volitelně zobrazované jméno odesílatele.
//! Node ani anchor obsah nevidí; rámec je součástí šifrovaného plaintextu.

/// Značka rámce. Plaintext, který s ní nezačíná, je holý text (starší klienti, CLI).
const FRAME_V1: u8 = 0x01;
/// Nejdelší jméno v bajtech UTF-8.
pub const MAX_NAME_BYTES: usize = 64;
/// Nejdelší jméno ve znacích.
pub const MAX_NAME_CHARS: usize = 32;

/// Rozbalený obsah zprávy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Content {
    /// Jméno, které si odesílatel nastavil. Je to jeho tvrzení, ne ověřený fakt (ověřuje se bezpečnostním číslem).
    pub name: Option<String>,
    pub text: String,
}

/// Zobrazované jméno: ořezané, bez řídicích a směrových znaků, nejvýše `MAX_NAME_CHARS` znaků. `None`, je-li prázdné.
pub fn clean_name(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}'))
        .take(MAX_NAME_CHARS)
        .collect();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_owned())
}

/// Sestaví plaintext: `0x01 ‖ délka jména (1 B) ‖ jméno ‖ text`. Bez jména a s textem, který nezačíná značkou,
/// zůstane holý text (kompatibilní se staršími klienty).
pub fn pack(name: Option<&str>, text: &str) -> Vec<u8> {
    let name = name.and_then(clean_name);
    if name.is_none() && text.as_bytes().first() != Some(&FRAME_V1) {
        return text.as_bytes().to_vec();
    }
    let name = name.unwrap_or_default();
    let mut out = Vec::with_capacity(2 + name.len() + text.len());
    out.push(FRAME_V1);
    // clean_name drží jméno pod 32 znaky; i ve 4bajtových znacích by to bylo víc než MAX_NAME_BYTES, proto zkrácení
    let bytes = name.as_bytes();
    let mut end = bytes.len().min(MAX_NAME_BYTES);
    while !name.is_char_boundary(end) {
        end -= 1;
    }
    out.push(u8::try_from(end).unwrap_or(0));
    out.extend_from_slice(&bytes[..end]);
    out.extend_from_slice(text.as_bytes());
    out
}

/// Rozbalí plaintext. Cokoli, co není platný rámec, je holý text; chybné jméno se zahodí, zpráva ne.
pub fn unpack(plaintext: &[u8]) -> Content {
    if let Some((&FRAME_V1, rest)) = plaintext.split_first() {
        if let Some((&len, rest)) = rest.split_first() {
            let len = usize::from(len);
            if len <= MAX_NAME_BYTES && rest.len() >= len {
                let (name, text) = rest.split_at(len);
                return Content { name: std::str::from_utf8(name).ok().and_then(clean_name), text: String::from_utf8_lossy(text).into_owned() };
            }
        }
    }
    Content { name: None, text: String::from_utf8_lossy(plaintext).into_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_text_round_trip() {
        let c = unpack(&pack(Some("Alice"), "ahoj"));
        assert_eq!(c, Content { name: Some("Alice".into()), text: "ahoj".into() });
    }

    #[test]
    fn plain_text_stays_plain_and_legacy_text_is_read_as_is() {
        assert_eq!(pack(None, "ahoj"), b"ahoj");
        assert_eq!(unpack(b"ahoj"), Content { name: None, text: "ahoj".into() });
        assert_eq!(pack(Some("   "), "ahoj"), b"ahoj");
    }

    #[test]
    fn text_starting_with_the_marker_is_escaped_by_a_frame() {
        let c = unpack(&pack(None, "\u{1}tajné"));
        assert_eq!(c, Content { name: None, text: "\u{1}tajné".into() });
    }

    #[test]
    fn malformed_frames_fall_back_to_text() {
        assert_eq!(unpack(&[1]).name, None);
        assert_eq!(unpack(&[1, 9, b'a']).name, None);
        assert_eq!(unpack(&[1, 200, b'a']).name, None);
        // neplatné UTF-8 ve jménu: zpráva projde, jméno ne
        assert_eq!(unpack(&[1, 2, 0xff, 0xfe, b'x']), Content { name: None, text: "x".into() });
    }

    #[test]
    fn names_are_cleaned_and_limited() {
        assert_eq!(clean_name("  Jan\u{202E}Novák \n"), Some("JanNovák".into()));
        assert_eq!(clean_name(&"x".repeat(100)).map(|n| n.chars().count()), Some(MAX_NAME_CHARS));
        assert_eq!(clean_name("\u{200B}\t"), None);
        let wide = "😀".repeat(40);
        let c = unpack(&pack(Some(&wide), "t"));
        assert!(c.name.is_some_and(|n| n.len() <= MAX_NAME_BYTES));
        assert_eq!(c.text, "t");
    }
}
