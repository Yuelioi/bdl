use std::{env, fs, path::PathBuf};

fn main() {
    const PATH: &str = "src/error-catalog.json";
    println!("cargo:rerun-if-changed={PATH}");
    let catalog: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&fs::read(PATH).expect("read error catalog"))
            .expect("valid error catalog");
    let mut variants = String::new();
    let mut names = String::new();
    for (code, entry) in &catalog {
        assert!(
            code.starts_with("bdl.")
                && code
                    .split(['.', '_'])
                    .all(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_lowercase())),
            "invalid code {code}"
        );
        for (field, text) in entry.as_object().expect("catalog entry") {
            assert!(
                matches!(field.as_str(), "message" | "advice"),
                "unknown {code}.{field}"
            );
            for lang in ["en", "zh-CN"] {
                assert!(
                    text[lang].as_str().is_some_and(|s| !s.trim().is_empty()),
                    "missing {code}.{field}.{lang}"
                );
            }
        }
        if entry.get("message").is_none() {
            assert!(entry.get("advice").is_some(), "empty category {code}");
            continue;
        }
        let mut key = code.as_str();
        while catalog.get(key).and_then(|e| e.get("advice")).is_none() {
            key = key
                .rsplit_once('.')
                .unwrap_or_else(|| panic!("no solution for {code}"))
                .0;
        }
        let variant: String = code[4..]
            .split(['.', '_'])
            .map(|word| {
                let mut word = word.to_owned();
                word[..1].make_ascii_uppercase();
                word
            })
            .collect();
        variants.push_str(&format!("#[serde(rename = {code:?})] {variant},\n"));
        names.push_str(&format!("Self::{variant} => {code:?},\n"));
    }
    let generated = format!(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\npub enum ErrorCode {{ {variants} }}\nimpl ErrorCode {{ pub const fn as_str(self) -> &'static str {{ match self {{ {names} }} }} }}"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("error_codes.rs"),
        generated,
    )
    .expect("write Rust error codes");
}
