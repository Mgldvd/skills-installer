use crate::error::AppError;

/// Curated palette for the application theme (pink, red/coral, orange, amber,
/// green, teal, cyan, blue, indigo, violet). Also used to assign
/// deterministic colors to legacy groups that have none.
pub const CURATED_PALETTE: &[(&str, &str)] = &[
    ("pink", "#F43F75"),
    ("coral", "#F05252"),
    ("orange", "#F97316"),
    ("amber", "#F59E0B"),
    ("green", "#22C55E"),
    ("teal", "#14B8A6"),
    ("cyan", "#06B6D4"),
    ("blue", "#3B82F6"),
    ("indigo", "#6366F1"),
    ("violet", "#A855F7"),
];

/// Validates and normalizes a color string to uppercase `#RRGGBB`. Rejects
/// anything that is not exactly a 6-digit hex triplet — in particular this
/// rejects `url(...)`, `expression(...)`, `var(...)`, `javascript:...`, and
/// any other CSS value, since those are exactly the injection vectors a
/// "custom color" field would otherwise open up when the value is later
/// interpolated into a `<style>` custom property on the frontend.
pub fn validate_and_normalize_color(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    let hex = trimmed.strip_prefix('#').ok_or_else(|| {
        AppError::Validation(format!("group color \"{raw}\" must start with '#'"))
    })?;

    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Validation(format!(
            "group color \"{raw}\" must be exactly 6 hexadecimal digits, e.g. #F43F75"
        )));
    }

    Ok(format!("#{}", hex.to_ascii_uppercase()))
}

/// Deterministic (never random) color assignment for legacy groups that
/// don't specify a color — stable across runs because it's derived purely
/// from the group id via a simple FNV-1a hash, not from insertion order or
/// wall-clock time.
pub fn deterministic_color_for_group_id(group_id: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in group_id.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    let index = (hash as usize) % CURATED_PALETTE.len();
    CURATED_PALETTE[index].1.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_normalizes_valid_hex() {
        assert_eq!(validate_and_normalize_color("#f43f75").unwrap(), "#F43F75");
        assert_eq!(validate_and_normalize_color("#ABCDEF").unwrap(), "#ABCDEF");
    }

    #[test]
    fn rejects_css_injection_vectors() {
        for bad in [
            "url(javascript:alert(1))",
            "expression(alert(1))",
            "var(--accent)",
            "javascript:alert(1)",
            "red",
            "#GGGGGG",
            "#12345",
            "#1234567",
            "",
        ] {
            assert!(
                validate_and_normalize_color(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn deterministic_color_is_stable_across_calls() {
        let a = deterministic_color_for_group_id("legacy-group");
        let b = deterministic_color_for_group_id("legacy-group");
        assert_eq!(a, b);
        assert!(CURATED_PALETTE.iter().any(|(_, hex)| *hex == a));
    }

    #[test]
    fn deterministic_color_varies_by_id() {
        let a = deterministic_color_for_group_id("alpha");
        let b = deterministic_color_for_group_id("bravo-team");
        // Not a strict guarantee for all inputs, but true for this pair and
        // demonstrates the function isn't a constant.
        assert_ne!(a, b);
    }
}
