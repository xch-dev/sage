use bigdecimal::BigDecimal;

pub fn encode_xch_amount(amount: u128) -> String {
    (BigDecimal::from(amount) / BigDecimal::from(1_000_000_000_000u128)).to_string()
}
