mod build_app;
mod setup;

#[cfg(test)]
mod api_tests {
    use crate::{build_app::build_app, setup};
    use http::StatusCode;
    use axum_test::{TestServer, TestServerConfig};
    use serde_json::json;
    use tokio::sync::Mutex;
    use lazy_static::lazy_static;
    use cookie::Cookie;
    
    lazy_static! {
        static ref INIT: Mutex<bool> = Mutex::new(false);
    }

    async fn initialize() {
        let mut lock = INIT.lock().await;
        if !*lock {
            setup::prepare_database().await;
            *lock=true;
        };
    }

    async fn login(server: &TestServer){
        let req = server.post("/api/login16502948954868423048")
            .expect_success()
            .form(&json!({
                "username": "user",
                "password": "user",
                "remember": 0
            }));
        
        let _res = req.await;
        //println!("{:?}", _res);
    }

    #[tokio::test]
    async fn test_auth_without_login(){
        initialize().await;
        let server = TestServer::new(build_app().await).unwrap();

        let req = server.post("/api/fetch_files17384435655166834659")
            .expect_failure()
            .form(&json!({
                "db_index": "0",
                "count": 1
            }));
        
        let _res = req.await;

        //assert!(res.status_code()==StatusCode::FORBIDDEN, 
        //    "Request should not be permitted, but response code is {}, message is {}", res.status_code(), res.text());
    }

    #[tokio::test]
    async fn test_auth_with_malformed_cookie(){
        initialize().await;
        let server = TestServer::new(build_app().await).unwrap();
        let cookie = Cookie::new("session", "qB4yBho4oHbMF8bTjT9m8Q");

        let req = server.post("/api/fetch_files17384435655166834659")
            .expect_failure()
            .add_cookie(cookie)
            .form(&json!({
                "db_index": "0",
                "count": 1
            }));
        
        let _res = req.await;

        //assert!(res.status_code()==StatusCode::FORBIDDEN, 
        //    "Request should not be permitted, but response code is {}, message is {}", res.status_code(), res.text());
    }

    #[tokio::test]
    async fn test_auth_with_login(){
        initialize().await;
        let config = TestServerConfig::builder()
            .save_cookies()
            .build();
        let server = TestServer::new_with_config(build_app().await, config).unwrap();
        login(&server).await;

        let req = server.post("/api/fetch_files17384435655166834659")
            .form(&json!({
                "db_index": "0",
                "count": 1
            }));
        
        let res = req.await;

        //println!("{:?}", res);
        //println!("{:?}", res.text());

        assert!(res.status_code()==StatusCode::OK, 
            "Request failed, response code is {}, message is {}", res.status_code(), res.text());
    }

    #[tokio::test]
    async fn test_fetch_files(){
        initialize().await;
        let config = TestServerConfig::builder()
            .save_cookies()
            .build();
        let server = TestServer::new_with_config(build_app().await, config).unwrap();
        login(&server).await;

        let req = server.post("/api/fetch_files17384435655166834659")
            .form(&json!({
                "db_index": "0",
                "count": 1
            }));
        
        let res = req.await;

        //println!("{:?}", res);
        //println!("{:?}", res.text());

        //TODO: check response
        assert!(res.status_code()==StatusCode::OK, 
            "Request failed, response code is {}, message is {}", res.status_code(), res.text());
    }
}
