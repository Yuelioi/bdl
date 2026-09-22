use crate::{BpiError, BpiResult};

pub type CookiePair = (String, String);

pub fn parse_cookie_header(cookie_header: &str) -> BpiResult<Vec<CookiePair>> {
    let mut pairs = Vec::new();

    for segment in cookie_header
        .split(';')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
    {
        let Some((key, value)) = segment.split_once('=') else {
            return Err(BpiError::invalid_parameter(
                "cookie",
                "cookie segment must contain '='",
            ));
        };

        let key = key.trim();
        if key.is_empty() {
            return Err(BpiError::invalid_parameter(
                "cookie",
                "cookie key cannot be empty",
            ));
        }

        pairs.push((key.to_string(), value.trim().to_string()));
    }

    if pairs.is_empty() {
        return Err(BpiError::invalid_parameter(
            "cookie",
            "cookie string cannot be empty",
        ));
    }

    Ok(pairs)
}

pub fn format_cookie_pairs(pairs: &[CookiePair]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiError;

    #[test]
    fn parse_cookie_header_returns_named_pairs() -> Result<(), BpiError> {
        let pairs = parse_cookie_header("SESSDATA=session; bili_jct=csrf")?;

        assert_eq!(
            pairs,
            vec![
                ("SESSDATA".to_string(), "session".to_string()),
                ("bili_jct".to_string(), "csrf".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn parse_cookie_header_rejects_invalid_segments() {
        let err = parse_cookie_header("SESSDATA").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "cookie",
                ..
            }
        ));
    }

    #[test]
    fn parse_cookie_header_rejects_empty_input() {
        let err = parse_cookie_header(" ; ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "cookie",
                ..
            }
        ));
    }

    #[test]
    fn format_cookie_pairs_returns_deterministic_header() {
        let pairs = vec![
            ("SESSDATA".to_string(), "session".to_string()),
            ("bili_jct".to_string(), "csrf".to_string()),
        ];

        assert_eq!(
            format_cookie_pairs(&pairs),
            "SESSDATA=session; bili_jct=csrf"
        );
    }
}
