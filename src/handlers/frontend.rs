use crate::errors::GitResult;
use crate::package_index::PackageIndex;
use actix_web::{get, web, HttpRequest};
use askama::Template;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Template)]
#[template(path = "landing.html")]
pub struct LandingTemplate<'a> {
    name: &'a str,
    packages: Vec<(String, String)>,
    all: bool,
    limit: usize,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    name: &'a str,
    token: &'a str,
}

#[derive(Deserialize)]
pub struct Query {
    all: Option<bool>,
}

#[get("/")]
pub async fn landing(
    query: web::Query<Query>,
    index: web::Data<Mutex<PackageIndex>>,
) -> GitResult<LandingTemplate<'static>> {
    let all = query.all.unwrap_or(false);
    let limit = if all { None } else { Some(25) };

    let entries = {
        let index = index.lock().unwrap();
        index.get_publishes(limit)?
    };

    Ok(LandingTemplate {
        name: "Estuary",
        packages: entries,
        all,
        limit: limit.unwrap_or(0),
    })
}

#[get("/me")]
pub async fn login(_req: HttpRequest) -> LoginTemplate<'static> {
    LoginTemplate {
        name: "Estuary",
        token: "0000", // TODO: implement proper auth
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers;
    use actix_web::http::StatusCode;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_landing_ok_empty() {
        let data_root = test_helpers::get_data_root();
        let settings = test_helpers::get_test_settings(&data_root.path());
        let package_index = test_helpers::get_test_package_index(&settings.index_dir);
        let mut app = test::init_service(
            App::new()
                .app_data(package_index.clone())
                .app_data(settings.clone())
                .configure(crate::handlers::configure_routes),
        )
        .await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
    }

    #[actix_rt::test]
    async fn test_login() {
        let data_root = test_helpers::get_data_root();
        let settings = test_helpers::get_test_settings(&data_root.path());
        let package_index = test_helpers::get_test_package_index(&settings.index_dir);

        let mut app = test::init_service(
            App::new()
                .app_data(package_index.clone())
                .app_data(settings.clone())
                .configure(crate::handlers::configure_routes),
        )
        .await;

        let req = test::TestRequest::get().uri("/me").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
    }
}
