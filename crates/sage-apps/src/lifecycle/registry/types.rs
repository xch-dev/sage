use serde::{Deserialize, Serialize};

use crate::{SageApp, UserSageApp};

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedUserSageApp {
    app: UserSageApp,
}

impl TryFrom<&SageApp> for PersistedUserSageApp {
    type Error = anyhow::Error;

    fn try_from(app: &SageApp) -> anyhow::Result<Self> {
        let user_app = app
            .as_user()
            .ok_or_else(|| anyhow::anyhow!("not a user app"))?;

        Self::try_from(user_app)
    }
}

impl TryFrom<&UserSageApp> for PersistedUserSageApp {
    type Error = anyhow::Error;

    fn try_from(user_app: &UserSageApp) -> anyhow::Result<Self> {
        Ok(Self {
            app: user_app.clone_durable(),
        })
    }
}

impl TryFrom<PersistedUserSageApp> for UserSageApp {
    type Error = anyhow::Error;

    fn try_from(persisted: PersistedUserSageApp) -> anyhow::Result<Self> {
        Ok(persisted.app)
    }
}
