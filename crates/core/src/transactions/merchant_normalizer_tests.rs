//! Tests for merchant normalization (D-13).

#[cfg(test)]
mod tests {
    use crate::transactions::merchant_normalizer::normalize_merchant;

    #[test]
    fn wholefds_grp() {
        assert_eq!(normalize_merchant("WHOLEFDS GRP #10403"), "wholefds grp #");
    }

    #[test]
    fn starbucks_double_space() {
        assert_eq!(
            normalize_merchant("STARBUCKS  STORE 12345"),
            "starbucks store #"
        );
    }

    #[test]
    fn amazon_leading_trailing_whitespace() {
        assert_eq!(normalize_merchant("  Amazon.com  "), "amazon.com");
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_merchant(""), "");
    }

    #[test]
    fn unicode_cafe() {
        assert_eq!(normalize_merchant("CAFÉ DU MONDE 99"), "café du monde #");
    }

    #[test]
    fn multiple_digit_runs() {
        assert_eq!(normalize_merchant("UBER 1 USD 2"), "uber # usd #");
    }
}
