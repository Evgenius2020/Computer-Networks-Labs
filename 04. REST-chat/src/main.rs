extern crate futures;
extern crate hyper;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use std::sync::{Arc, Mutex};

mod users_manager;
use users_manager::UsersManager;

const FILENAME: &str = "target/db.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    id: usize,
    username: String,
    online: bool,
}

#[derive(Serialize)]
pub struct LoginResult {
    id: usize,
    username: String,
    online: bool,
    token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
}

#[derive(Serialize)]
pub struct UsersResult {
    users: Vec<User>,
}

type Fut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn on_request_received(req: Request<Body>, um: Arc<Mutex<UsersManager>>) -> Fut {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::POST, "/login") => {
            let res = req.into_body().concat2().map(move |chunk| {
                let body = chunk.iter().cloned().collect::<Vec<u8>>();
                let login_request: LoginRequest =
                    serde_json::from_str(&String::from_utf8(body).unwrap()).unwrap();

                let mut um = um.lock().unwrap();

                match (*um).get_or_create(&login_request.username) {
                    Some((user, created)) => {
                        if created {
                            um.save(FILENAME);
                        }

                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                serde_json::to_string(&um.generate_login_result(&user)).unwrap(),
                            )).unwrap()
                    }
                    None => Response::builder()
                        .header(
                            "WWW-Authenticate",
                            "Token realm='Username is already in use'",
                        ).status(StatusCode::UNAUTHORIZED)
                        .body(Body::from(""))
                        .unwrap(),
                }
            });
            return Box::new(res);
        }
        (&Method::POST, "/logout") => {
            let mut um = um.lock().unwrap();
            let token = get_token(req);
            match check_token(token.clone(), &um) {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    (*um).logout(&token.unwrap());
                    um.save(FILENAME);
                    let res = Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{ "message": "bye!" }"#))
                        .unwrap();
                    return Box::new(future::ok(res));
                }
            }
        }
        (&Method::GET, "/users") => {
            let mut um = um.lock().unwrap();
            let token = get_token(req);
            match check_token(token.clone(), &um) {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    return Box::new(future::ok(
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                serde_json::to_string(&(*um).generate_users_result()).unwrap(),
                            )).unwrap(),
                    ))
                }
            }
        }

        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
}

fn get_token(req: hyper::Request<hyper::Body>) -> Option<String> {
    match req.into_parts().0.headers.get("Authorization") {
        Some(auth_header) => {
            let words = auth_header.to_str().unwrap().split(" ");
            if (words.clone().count() != 2) || (words.clone().nth(0).unwrap() != "Token") {
                return None;
            }
            return Some(words.clone().nth(1).unwrap().to_string());
        }
        None => None,
    }
}

fn check_token(token: Option<String>, um: &UsersManager) -> Option<hyper::Response<hyper::Body>> {
    match token {
        Some(token) => match um.get_id_by_token(&token) {
            Some(_) => None,
            None => Some(
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from(""))
                    .unwrap(),
            ),
        },
        None => Some(
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(""))
                .unwrap(),
        ),
    }
}

fn main() {
    let addr = ([127, 0, 0, 1], 1337).into();

    hyper::rt::run(future::lazy(move || {
        let um = match UsersManager::load(FILENAME) {
            Some(um) => um,
            None => UsersManager::new(),
        };
        let um = Arc::new(Mutex::new(um));

        let server = Server::bind(&addr)
            .serve(move || {
                let um = um.clone();
                service_fn(move |req| on_request_received(req, um.clone()))
            }).map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }));
}
