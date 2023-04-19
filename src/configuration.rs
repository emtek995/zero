use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub database_password: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "mongodb+srv://{}:{}@{}",
            self.database_name, self.database_password, self.host
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
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
        .build()?;

    settings.try_deserialize()
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
