pub(crate) fn normalized_non_empty_string(
    value: impl Into<String>,
    label: &str,
) -> anyhow::Result<String> {
    let value = value.into().trim().to_string();

    if value.is_empty() {
        anyhow::bail!("{label} cannot be empty");
    }

    Ok(value)
}

pub(crate) fn normalized_optional_string(
    value: Option<impl Into<String>>,
) -> Option<String> {
    value
        .map(|value| value.into().trim().to_string())
        .filter(|value| !value.is_empty())
}
