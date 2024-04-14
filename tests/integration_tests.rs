mod build_app;

#[cfg(test)]
mod api_tests {
    use tower::ServiceExt;

    use crate::build_app::build_app;
    use axum::body::{Body, HttpBody, Bytes, to_bytes};
    use http::{header, Request, StatusCode};
    use axum_test::{TestServer, TestRequest};
    use serde_json::json;

    #[tokio::test]
    async fn test_fetch_files_without_login(){
        let server = TestServer::new(build_app().await).unwrap();

        let req = server.post("/api/fetch_files17384435655166834659")
            .form(&json!({
                "db_index": "0",
                "count": 1
            }));
        
        let res = req.await;

        println!("{:?}", res);
        println!("{:?}", res.text());

        assert!(res.status_code()==StatusCode::OK, 
            "Request failed, response code is {}, message is {}", res.status_code(), res.text());
    }

    #[tokio::test]
    async fn test_fetch_files_with_malformed_cookie(){

    }

    #[tokio::test]
    async fn test_fetch_files_with_login(){

    }
}
