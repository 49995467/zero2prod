use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
 async fn main() -> std::io::Result<()>{
    let subscriber=get_subscriber("zero2prod".into(),"info".into(),std::io::stdout);
    init_subscriber(subscriber);

    //将`log`中的记录导入到`trace`中
    // LogTracer::init().expect("Failed to set logger.");
//如果没有设置RUST_LOG环境变量，则默认为info,输出所有`info`及以上级别的跨度
//     let env_filter=EnvFilter::try_from_default_env().unwrap_or_else(|_| {EnvFilter::new("info")});
//     let formatting_layer=BunyanFormattingLayer::new("zero2prod".into(),
                                                    //将格式化的跨度输出到标准输出
                                                    // std::io::stdout);
    // `with`方法由`SubscribeExt`提供,可以扩展`tracing_subscriber`的`Subscriber`类型
    // let subsciber=Registry::default()
    //     .with(env_filter)
    //     .with(JsonStorageLayer)
    //     .with(formatting_layer);
    // `set_global_default`方法可以用于指定处理跨度的订阅器
    // set_global_default(subsciber).expect("Failed to set subscriber.");
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // let utc_time=Utc::now();
    // println!("UTC:{}",utc_time);
    // println!("DateTime:{}",utc_time.with_timezone(&Local));
    //如果不能读取配置文件，则发生panic
    let configuration=get_configuration().expect("Failed to read configuration.");
    let connection_pool=PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    // let connection=PgConnection::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");
    let address=format!("127.0.0.1:{}",configuration.application_port);
     let listener=TcpListener::bind(address)?;
    println!("Listening on:{}",listener.local_addr()?);
     run(listener,connection_pool)?.await
 }
