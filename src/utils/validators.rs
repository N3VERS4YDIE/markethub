use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use validator::ValidationError;

pub static SLUG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("Slug regex should compile"));

pub fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if SLUG_REGEX.is_match(value) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_slug"))
    }
}

pub fn validate_shipping_address(value: &Value) -> Result<(), ValidationError> {
    if let Some(obj) = value.as_object() {
        if obj.is_empty() {
            return Err(ValidationError::new("empty_address"));
        }
        return Ok(());
    }
    Err(ValidationError::new("invalid_address"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_regex_matches_valid() {
        assert!(SLUG_REGEX.is_match("valid-slug-123"));
        assert!(!SLUG_REGEX.is_match("Invalid Slug"));
    }

    #[test]
    fn shipping_address_validation() {
        let valid = serde_json::json!({"line1": "123 Main", "city": "NY"});
        assert!(validate_shipping_address(&valid).is_ok());

        let invalid = serde_json::json!({});
        assert!(validate_shipping_address(&invalid).is_err());

        let not_obj = serde_json::json!("string");
        assert!(validate_shipping_address(&not_obj).is_err());
    }
}
