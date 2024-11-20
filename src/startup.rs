use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web,App,HttpServer};
use actix_web::middleware::Logger;
use crate::routes::{health_check, subscribe};
use sqlx::PgPool;
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;

pub fn run(listener:TcpListener, db_pool:PgPool) -> Result<Server,std::io::Error> {
    // 将连接池包装在一个智能指针中，以便可以在多个处理程序之间共享它，其本质上是一个引用计数指针Arc
    let db_pool=Data::new(db_pool);
    // let connection=Data::new(connection);
    let server=HttpServer::new(move|| {
        App::new()
            //将中间件通过`wrap`方法添加到App中
            // .wrap(Logger::default())
            // 使用`TracingLogger`替换`Logger`
            .wrap(TracingLogger::default())
            // .route("/", web::get().to(greet))
            // .route("/{name}",web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions",web::post().to(subscribe))
            .app_data(db_pool.clone())
            // .app_data(connection.clone())
    })
        .listen(listener)?
        .run();
    // .await
    Ok(server)
}