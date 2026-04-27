mod common;

use common::sample_manifest_file;
use sage_apps::lifecycle::limits::{MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES};
use sage_apps::lifecycle::manifest::{
    validate_manifest_file_path, validate_manifest_files, validate_sha256_hex,
};
use sage_apps::types::{
    SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
    SageRequestedPermissions,
};

fn sample_manifest() -> SageAppPackageManifest {
    SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: "Test App".to_string(),
        version: "1.0.0".to_string(),
        permissions: SageRequestedPermissions::empty(),
        files: vec![sample_manifest_file("dist/index.html", 123)],
        entry: Some("dist/index.html".to_string()),
        icon: Some("dist/icon.png".to_string()),
        author: None,
        donation: None,
    })
    .unwrap()
}

#[test]
fn validate_manifest_file_path_accepts_normal_relative_path() {
    validate_manifest_file_path("dist/index.html").unwrap();
}

#[test]
fn validate_manifest_file_path_rejects_absolute_path() {
    assert!(validate_manifest_file_path("/etc/passwd").is_err());
}

#[test]
fn validate_manifest_file_path_rejects_parent_traversal() {
    assert!(validate_manifest_file_path("../secret.txt").is_err());
}

#[test]
fn validate_manifest_file_path_rejects_current_dir_segment() {
    assert!(validate_manifest_file_path("./index.html").is_err());
    assert!(validate_manifest_file_path("dist/./index.html").is_err());
}

#[test]
fn validate_manifest_file_path_rejects_empty_segment() {
    assert!(validate_manifest_file_path("dist//index.html").is_err());
}

#[test]
fn validate_manifest_file_path_rejects_backslashes() {
    assert!(validate_manifest_file_path(r"dist\index.html").is_err());
}

#[test]
fn validate_sha256_hex_accepts_valid_hash() {
    validate_sha256_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .unwrap();
}

#[test]
fn validate_sha256_hex_rejects_invalid_hash() {
    assert!(validate_sha256_hex("not-a-sha").is_err());
}

#[test]
fn validate_manifest_files_rejects_empty_list() {
    let err = validate_manifest_files(&[]).unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn validate_manifest_files_rejects_duplicate_paths() {
    let files = vec![
        sample_manifest_file("dist/index.html", 1),
        sample_manifest_file("dist/index.html", 2),
    ];

    let err = validate_manifest_files(&files).unwrap_err();
    assert!(err.to_string().contains("duplicate manifest file path"));
}

#[test]
fn validate_manifest_files_rejects_invalid_nested_path() {
    let files = vec![sample_manifest_file("dist//index.html", 1)];

    let err = validate_manifest_files(&files).unwrap_err();
    assert!(err.to_string().contains("manifest file path is invalid"));
}

#[test]
fn validate_manifest_files_rejects_file_count_over_limit() {
    let files: Vec<_> = (0..=MAX_APP_FILE_COUNT)
        .map(|i| sample_manifest_file(&format!("dist/file-{i}.txt"), 1))
        .collect();

    let err = validate_manifest_files(&files).unwrap_err();
    assert!(err.to_string().contains("exceeds limit"));
}

#[test]
fn validate_manifest_files_rejects_total_size_over_limit() {
    let files = vec![
        sample_manifest_file("dist/a.bin", MAX_APP_TOTAL_SIZE_BYTES),
        sample_manifest_file("dist/b.bin", 1),
    ];

    let err = validate_manifest_files(&files).unwrap_err();
    assert!(err.to_string().contains("manifest total size"));
    assert!(err.to_string().contains("exceeds limit"));
}

#[test]
fn validate_manifest_files_returns_total_size_when_valid() {
    let files = vec![
        sample_manifest_file("dist/index.html", 100),
        sample_manifest_file("dist/icon.png", 23),
    ];

    let total = validate_manifest_files(&files).unwrap();
    assert_eq!(total, 123);
}

#[test]
fn manifest_rejects_blank_name() {
    let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: "   ".to_string(),
        version: "1.0.0".to_string(),
        permissions: SageRequestedPermissions::empty(),
        files: vec![sample_file()],
        entry: Some("index.html".to_string()),
        icon: Some("icon.png".to_string()),
        author: None,
        donation: None,
    })
    .unwrap_err();

    assert!(err.to_string().contains("name cannot be empty"));
}

#[test]
fn manifest_rejects_blank_version() {
    let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: "Test".to_string(),
        version: "   ".to_string(),
        permissions: SageRequestedPermissions::empty(),
        files: vec![sample_file()],
        entry: Some("index.html".to_string()),
        icon: Some("icon.png".to_string()),
        author: None,
        donation: None,
    })
    .unwrap_err();

    assert!(err.to_string().contains("version cannot be empty"));
}

#[test]
fn manifest_total_size_is_computed() {
    let manifest = sample_manifest();
    let total = manifest.total_bytes().unwrap();

    assert_eq!(total, 123);
}

fn sample_file() -> SageAppManifestFile {
    SageAppManifestFile {
        path: "index.html".into(),
        sha256: "a".repeat(64),
        size: 123,
    }
}
