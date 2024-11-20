//! tests/health_check.rs

use std::net::TcpListener;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSetting};
use zero2prod::startup;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;

//使用`once_cell`确保tracing只初始化一次
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    }else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    // 准备
    let app = spawn_app().await;

    println!("{}", app.address);

    //需要引入`reqWest`对应用程序执行HTTP请求
    let client = reqwest::Client::new();
    //执行
    let response = client.get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    //断言
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    //准备
    let app = spawn_app().await;
    // let configuration=get_configuration().expect("Failed to read configuration.");
    // let connection_string=configuration.database.connection_string();
    // 为了调用`PgConnection::connect`,`Connection`trait必须位于作用域内
    // let mut connection=PgConnection::connect(&connection_string)
    //     .await
    //     .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    //执行
    let body = "name=le%20guin&email=49995467%40qq.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    //断言
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "49995467@qq.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {

    //准备
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=49995467%40qq.com", "missing the name"),
        ("", "missing both name and email")
    ];

    //执行
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        //断言
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
async fn spawn_app() -> TestApp {
    // let subscriber=get_subscriber("test".into(),"debug".into());
    // init_subscriber(subscriber);
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    // let connection_pool=PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    //启动服务器作为后台任务,
    // tokio::spawn返回一个指向spawned future 的handle
    //但是这里没有用到它,因为这是非绑定的let用法
    let _ = tokio::spawn(server);

    //将应用程序地址返回给调用者
    // format!("http://127.0.0.1:{}",port)
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSetting) -> PgPool {
    //创建数据库

    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection.execute(format!(
        r#"
        CREATE DATABASE "{};"
        "#,
        config.database_name)
        .as_str()
    ).await
        .expect("Failed to create database.");

    //这个修改后的代码在连接到数据库之前添加了一个循环，确保数据库已经成功创建。
    // 如果连接失败，会等待一秒钟再重试。
    // 这样可以避免因为数据库创建和连接之间的时间差导致的错误
    let connection_pool = loop {
        match PgPool::connect(&config.connection_string().expose_secret()).await {
            Ok(pool) => break pool,
            Err(_) => {
                //等待Postgres准备好
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };
    // 迁移数据库
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");
    connection_pool
}