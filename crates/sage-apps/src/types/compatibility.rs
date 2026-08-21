use semver::Version;
use serde::Serialize;
use specta::Type;
use tauri::AppHandle;

use crate::SageAppManifestSageVersion;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCompatibility {
    current_version: String,
    status: SageAppCompatibilityStatus,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SageAppCompatibilityStatus {
    Compatible,
    RequiresNewerSage {
        #[serde(rename = "minimumVersion")]
        minimum_version: String,
    },
    UntestedNewerSage {
        #[serde(rename = "testedMaxVersion")]
        tested_max_version: String,
    },
    Invalid {
        reason: String,
    },
}

impl SageAppCompatibility {
    pub fn evaluate(current: &Version, required: &SageAppManifestSageVersion) -> Self {
        let status = match parse_sage_version_range(required) {
            Ok((minimum, _)) if current < &minimum => {
                SageAppCompatibilityStatus::RequiresNewerSage {
                    minimum_version: minimum.to_string(),
                }
            }
            Ok((_, Some(tested_max))) if current > &tested_max => {
                SageAppCompatibilityStatus::UntestedNewerSage {
                    tested_max_version: tested_max.to_string(),
                }
            }
            Ok(_) => SageAppCompatibilityStatus::Compatible,
            Err(err) => SageAppCompatibilityStatus::Invalid {
                reason: err.to_string(),
            },
        };

        Self {
            current_version: current.to_string(),
            status,
        }
    }

    pub fn for_app(app: &AppHandle, required: &SageAppManifestSageVersion) -> Self {
        Self::evaluate(&app.package_info().version, required)
    }

    pub fn status(&self) -> &SageAppCompatibilityStatus {
        &self.status
    }

    pub fn ensure_installable(&self) -> anyhow::Result<()> {
        match &self.status {
            SageAppCompatibilityStatus::Compatible
            | SageAppCompatibilityStatus::UntestedNewerSage { .. } => Ok(()),
            SageAppCompatibilityStatus::RequiresNewerSage { minimum_version } => {
                anyhow::bail!(
                    "app requires Sage {minimum_version} or newer; current Sage version is {}",
                    self.current_version
                )
            }
            SageAppCompatibilityStatus::Invalid { reason } => {
                anyhow::bail!("app has an invalid Sage version requirement: {reason}")
            }
        }
    }
}

pub fn validate_sage_version_range(required: &SageAppManifestSageVersion) -> anyhow::Result<()> {
    parse_sage_version_range(required).map(|_| ())
}

fn parse_sage_version_range(
    required: &SageAppManifestSageVersion,
) -> anyhow::Result<(Version, Option<Version>)> {
    let minimum = Version::parse(&required.min).map_err(|err| {
        anyhow::anyhow!(
            "manifest sageVersion.min {:?} is not a valid semantic version: {err}",
            required.min
        )
    })?;

    let tested_max = required
        .tested_max
        .as_deref()
        .map(|value| {
            Version::parse(value).map_err(|err| {
                anyhow::anyhow!(
                    "manifest sageVersion.testedMax {value:?} is not a valid semantic version: {err}"
                )
            })
        })
        .transpose()?;

    if tested_max.as_ref().is_some_and(|tested| tested < &minimum) {
        anyhow::bail!(
            "manifest sageVersion.testedMax must be greater than or equal to sageVersion.min"
        );
    }

    Ok((minimum, tested_max))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(min: &str, tested_max: Option<&str>) -> SageAppManifestSageVersion {
        SageAppManifestSageVersion {
            min: min.to_string(),
            tested_max: tested_max.map(str::to_string),
        }
    }

    #[test]
    fn compatibility_covers_minimum_and_tested_max_boundaries() {
        let required = requirement("0.13.0", Some("0.14.0"));

        assert!(matches!(
            SageAppCompatibility::evaluate(&Version::parse("0.12.9").unwrap(), &required).status(),
            SageAppCompatibilityStatus::RequiresNewerSage { minimum_version }
                if minimum_version == "0.13.0"
        ));
        assert_eq!(
            SageAppCompatibility::evaluate(&Version::parse("0.13.0").unwrap(), &required).status(),
            &SageAppCompatibilityStatus::Compatible
        );
        assert_eq!(
            SageAppCompatibility::evaluate(&Version::parse("0.14.0").unwrap(), &required).status(),
            &SageAppCompatibilityStatus::Compatible
        );
        assert!(matches!(
            SageAppCompatibility::evaluate(&Version::parse("0.14.1").unwrap(), &required).status(),
            SageAppCompatibilityStatus::UntestedNewerSage { tested_max_version }
                if tested_max_version == "0.14.0"
        ));
    }

    #[test]
    fn invalid_ranges_are_reported_without_panicking() {
        for required in [
            requirement("not-a-version", None),
            requirement("0.13.0", Some("also-not-a-version")),
            requirement("1.0.0", Some("0.13.0")),
        ] {
            assert!(matches!(
                SageAppCompatibility::evaluate(&Version::parse("0.13.0").unwrap(), &required)
                    .status(),
                SageAppCompatibilityStatus::Invalid { .. }
            ));
        }
    }
}
