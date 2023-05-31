use anyhow::Result;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub database_password: Secret<String>,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "mongodb+srv://{}:{}@{}",
            self.database_name,
            self.database_password.expose_secret(),
            self.host
        ))
    }
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub client_secret: Secret<String>,
    pub sender_email: String,
}

impl EmailClientSettings {
    pub fn sender(&self) -> std::result::Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn client_secret(&self) -> Secret<String> {
        Secret::new(self.client_secret.expose_secret().clone())
    }
}

pub fn get_configuration() -> Result<Settings> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let base_path = std::env::current_dir().expect("Failed to get current directory");
    let base_path = base_path.to_str().expect("Failed to stringify base_path");

    let settings = config::Config::builder()
        .add_source(config::File::with_name(
            format!("{}/configuration/base", base_path).as_str(),
        ))
        .add_source(config::File::with_name(
            format!("{}/configuration/{}", base_path, environment.as_str()).as_str(),
        ))
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;

    settings.try_deserialize().map_err(|e| e.into())
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use `local` or `production`.",
                other
            )),
        }
    }
}
