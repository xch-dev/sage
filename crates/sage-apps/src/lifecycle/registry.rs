pub mod types;

use std::path::{Path, PathBuf};

use anyhow::Result as AnyResult;

use crate::system_apps::{SystemAppUsage, list_builtin_system_apps};
use crate::types::{ListedSageApp, SageApp};

pub fn apps_root(base_path: &Path) -> PathBuf {
    base_path.join("apps")
}

pub async fn list_installed_apps_internal(db: &crate::db::AppsDb) -> AnyResult<Vec<ListedSageApp>> {
    let mut apps = db.list_installed_apps().await?;

    for app in list_builtin_system_apps()? {
        if let SageApp::System(app) = app
            && app.usage() == SystemAppUsage::Standalone
        {
            apps.push(ListedSageApp::System(app));
        }
    }

    apps.sort_by_key(listed_app_sort_key);

    Ok(apps)
}

fn listed_app_sort_key(app: &ListedSageApp) -> String {
    match app {
        ListedSageApp::User(app) => app.common().name().to_lowercase(),
        ListedSageApp::System(app) => app.common().name().to_lowercase(),
        ListedSageApp::Corrupted(app) => app.id().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::lifecycle::install::{FakeInstallSource, install_app_from_source_for_test};
    use crate::types::{
        ListedSageApp, SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageGrantedPermissionsInput, SageRequestedPermissions, SharedSageApp, UserSageAppSource,
    };
    use tempfile::tempdir;

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(path, "a".repeat(64), size).unwrap()
    }

    fn sample_manifest(name: &str) -> SageAppPackageManifest {
        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: name.to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("index.html", 1)],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    async fn sample_app_named(
        base: &Path,
        db: &crate::db::AppsDb,
        app_id: &str,
        name: &str,
    ) -> SharedSageApp {
        let manifest = sample_manifest(name);
        let granted = SageGrantedPermissionsInput::new([], [], BTreeMap::new());

        let installed = install_app_from_source_for_test(
            base,
            granted,
            FakeInstallSource {
                manifest,
                app_id: app_id.into(),
                source: UserSageAppSource::url("https://example.com/app/").unwrap(),
            },
        )
        .await
        .unwrap();

        let storage_id = db
            .register_storage(installed.common().storage())
            .await
            .unwrap();

        let mut tx = db.begin_immediate().await.unwrap();

        let origin_row_id = tx
            .register_origin(installed.common().origin_id(), storage_id)
            .await
            .unwrap();

        tx.insert_user_app(&installed, storage_id, origin_row_id)
            .await
            .unwrap();

        tx.commit().await.unwrap();

        SharedSageApp::new(installed.into_sage_app())
    }

    fn without_system_apps(listed: Vec<ListedSageApp>) -> Vec<ListedSageApp> {
        listed
            .into_iter()
            .filter(|entry| !matches!(entry, ListedSageApp::System(_)))
            .collect()
    }

    #[tokio::test]
    async fn installed_apps_are_sorted_by_name() {
        let base = tempdir().unwrap();
        let db = crate::db::AppsDb::initialize(base.path()).await.unwrap();

        let _alpha = sample_app_named(base.path(), &db, "a", "Alpha").await;
        let _zeta = sample_app_named(base.path(), &db, "z", "Zeta").await;

        let listed = without_system_apps(list_installed_apps_internal(&db).await.unwrap());

        let names: Vec<_> = listed
            .into_iter()
            .map(|entry| match entry {
                ListedSageApp::User(app) => app.common().name().to_string(),
                ListedSageApp::System(app) => app.common().name().to_string(),
                ListedSageApp::Corrupted(app) => app.id().to_string(),
            })
            .collect();

        assert_eq!(names, vec!["Alpha".to_string(), "Zeta".to_string()]);
    }
}
