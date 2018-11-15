extern crate futures;
extern crate hyper;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use datatypes::LoginRequest;
use datatypes::MessageSendRequest;
use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use std::fmt::Error;
use std::sync::{Arc, Mutex};

mod data_manager;
mod datatypes;
use data_manager::DataManager;

const FILENAME: &str = "target/db.json";

type Fut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

enum MethodName {
    Login,
    Logout,
    Users,
    Messages,
}
fn on_request_received(req: Request<Body>, dm: Arc<Mutex<DataManager>>) -> Fut {
    let mut response = Response::new(Body::empty());

    match (req.method(), get_method_name(req.uri().path())) {
        (&Method::POST, Some(MethodName::Login)) => {
            let res = req.into_body().concat2().map(move |chunk| {
                let body = chunk.iter().cloned().collect::<Vec<u8>>();
                let login_request: LoginRequest =
                    serde_json::from_str(&String::from_utf8(body).unwrap()).unwrap();

                let mut dm = dm.lock().unwrap();

                match (*dm).get_or_create(&login_request.username) {
                    Some((user, created)) => {
                        if created {
                            dm.save(FILENAME);
                        }

                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                serde_json::to_string(&dm.generate_login_result(&user)).unwrap(),
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
        (&Method::POST, Some(MethodName::Logout)) => {
            let mut dm = dm.lock().unwrap();
            let token = get_token(&req.into_parts().0);
            match check_token(token.clone(), &dm) {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    (*dm).logout(&token.unwrap());
                    dm.save(FILENAME);
                    let res = Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{ "message": "bye!" }"#))
                        .unwrap();
                    return Box::new(future::ok(res));
                }
            }
        }
        (&Method::GET, Some(MethodName::Users)) => {
            let mut dm = dm.lock().unwrap();
            let head = req.into_parts().0;
            let token = get_token(&head);
            match check_token(token.clone(), &dm) {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    let res_json: String = match get_id_from_users_request(head.uri.to_string()) {
                        Some(user_id) => match (*dm).get_by_id(user_id) {
                            Some(user) => serde_json::to_string(user).unwrap(),
                            None => {
                                return Box::new(future::ok(
                                    Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(Body::from(""))
                                        .unwrap(),
                                ))
                            }
                        },
                        None => {
                            if head.uri == "/users" {
                                serde_json::to_string(&(*dm).generate_users_result()).unwrap()
                            } else {
                                return Box::new(future::ok(
                                    Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(""))
                                        .unwrap(),
                                ));
                            }
                        }
                    };
                    return Box::new(future::ok(
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(res_json))
                            .unwrap(),
                    ));
                }
            }
        }
        (&Method::POST, Some(MethodName::Messages)) => {
            let (head, body) = req.into_parts();
            let token = get_token(&head);
            let ch_tk = check_token(token.clone(), &dm.lock().unwrap());
            match ch_tk {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    let res = body.concat2().map(move |chunk| {
                        let body = chunk.iter().cloned().collect::<Vec<u8>>();
                        let message_send_request: MessageSendRequest =
                            serde_json::from_str(&String::from_utf8(body).unwrap()).unwrap();
                        let mut dm = dm.lock().unwrap();

                        let message_send_result =
                            &dm.add_message(message_send_request, &token.unwrap());
                        dm.save(FILENAME);
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                serde_json::to_string(message_send_result).unwrap(),
                            )).unwrap()
                    });
                    return Box::new(res);
                }
            }
        }
        (&Method::GET, Some(MethodName::Messages)) => {
            let (head, _) = req.into_parts();
            let uri = head.uri.to_string();
            let token = get_token(&head);
            let ch_tk = check_token(token.clone(), &dm.lock().unwrap());
            match ch_tk {
                Some(res) => return Box::new(future::ok(res)),
                None => {
                    let (offset, count) = match get_message_request_interval(uri) {
                        Ok((offset, count)) => (offset, count),
                        Err(_) => {
                            return Box::new(future::ok(
                                Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(""))
                                    .unwrap(),
                            ));
                        }
                    };
                    let mut dm = dm.lock().unwrap();

                    return Box::new(future::ok(
                        Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(
                                serde_json::to_string(
                                    &(*dm).generate_messages_result(offset, count),
                                ).unwrap(),
                            )).unwrap(),
                    ));
                }
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
}

// Authorization: Token <sometoken>
fn get_token(head: &hyper::http::request::Parts) -> Option<String> {
    match head.headers.get("Authorization") {
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

// /users/<id>
fn get_id_from_users_request(path: String) -> Option<usize> {
    if path == "/users" {
        return None;
    }

    let mut split = path.split("/");
    if (split.clone().count() != 3)
        || (split.nth(0).unwrap() != "")
        || (split.nth(0).unwrap() != "users")
    {
        return None;
    }
    match split.nth(0).unwrap().parse::<usize>() {
        Ok(id) => Some(id),
        Err(_) => None,
    }
}

// /messages?offset=<offset>&count=<count>
fn get_message_request_interval(path: String) -> Result<(usize, usize), Error> {
    if path == "/messages" {
        return Ok((0, 10));
    }
    let mut split = path.split("?");
    if (split.clone().count() == 0)
        || (split.clone().count() != 2)
        || (split.nth(0).unwrap() != "/messages")
    {
        return Err(Error);
    };

    let mut split = split.nth(0).unwrap().split("&");
    if split.clone().count() != 2 {
        return Err(Error);
    }

    let mut key_value = split.nth(0).unwrap().split("=");
    if (key_value.clone().count() != 2) || (key_value.nth(0).unwrap() != "offset") {
        return Err(Error);
    };
    let offset = match key_value.nth(0).unwrap().parse::<usize>() {
        Ok(offset) => offset,
        Err(_) => return Err(Error),
    };

    let mut key_value = split.nth(0).unwrap().split("=");
    if (key_value.clone().count() != 2) || (key_value.nth(0).unwrap() != "count") {
        return Err(Error);
    };
    let count = match key_value.nth(0).unwrap().parse::<usize>() {
        Ok(count) => {
            if count > 100 {
                return Err(Error);
            }
            count
        }
        Err(_) => return Err(Error),
    };

    Ok((offset, count))
}

fn check_token(token: Option<String>, dm: &DataManager) -> Option<hyper::Response<hyper::Body>> {
    match token {
        Some(token) => match dm.get_id_by_token(&token) {
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

fn get_method_name(path: &str) -> Option<MethodName> {
    match path.to_string().split("/").nth(1) {
        Some(root) => {
            if root == "login" {
                return Some(MethodName::Login);
            } else if root == "logout" {
                return Some(MethodName::Logout);
            } else if root == "users" {
                return Some(MethodName::Users);
            } else if root == "messages" {
                return Some(MethodName::Messages);
            }
            None
        }
        None => None,
    }
}

fn main() {
    let addr = ([127, 0, 0, 1], 1337).into();

    hyper::rt::run(future::lazy(move || {
        let dm = match DataManager::load(FILENAME) {
            Some(dm) => dm,
            None => DataManager::new(),
        };
        let dm = Arc::new(Mutex::new(dm));

        let server = Server::bind(&addr)
            .serve(move || {
                let dm = dm.clone();
                service_fn(move |req| on_request_received(req, dm.clone()))
            }).map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }));
}
