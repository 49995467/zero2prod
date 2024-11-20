use secrecy::{ExposeSecret,  SecretBox};
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSetting,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSetting {
    pub username: String,
    pub password: SecretBox<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSetting {
    pub fn connection_string(&self)->SecretBox<String>{
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,self.password.expose_secret(),self.host,self.port,self.database_name
        )))
    }

    pub fn connection_string_without_db(&self)->String{
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,self.password.expose_secret(),self.host,self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        // 从一个名为configuration.yaml的文件中读取配置
        .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;
    //尝试将其读取到的配置值反序列化为Settings结构体
    settings.try_deserialize::<Settings>()
}