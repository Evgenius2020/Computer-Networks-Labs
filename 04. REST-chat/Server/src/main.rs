extern crate futures;
extern crate hyper;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

mod data_manager;
mod datatypes;
mod parsing;

use data_manager::DataManager;
use datatypes::{LoginRequest, MessageSendRequest};
use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use parsing::{
    get_id_from_users_request, get_message_request_interval, get_method_name, get_token, MethodName,
};
use std::sync::{Arc, Mutex};

const FILENAME: &str = "target/db.json";

type Fut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn on_request_received(req: Request<Body>, dm: Arc<Mutex<DataManager>>) -> Fut {
    let mut response = Response::new(Body::empty());

    match (req.method(), get_method_name(req.uri().path())) {
        (&Method::POST, Some(MethodName::Login)) => {
            let res = req.into_body().concat2().map(move |chunk| {
                let body = chunk.iter().cloned().collect::<Vec<u8>>();
                let login_request: LoginRequest =
                    serde_json::from_str(&String::from_utf8(body).unwrap()).unwrap();

                let mut dm = dm.lock().unwrap();

                dm.update_online();
                dm.save(FILENAME);
                match (*dm).get_or_create(&login_request.username) {
                    Some(user) => {
                        dm.save(FILENAME);

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
            match check_token_error(token.clone(), &mut dm) {
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
            match check_token_error(token.clone(), &mut dm) {
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
            let ch_tk = check_token_error(token.clone(), &mut dm.lock().unwrap());
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
            let ch_tk = check_token_error(token.clone(), &mut dm.lock().unwrap());
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

fn check_token_error(
    token: Option<String>,
    dm: &mut DataManager,
) -> Option<hyper::Response<hyper::Body>> {
    match token {
        Some(token) => match dm.get_id_by_token(&token) {
            Some(id) => {
                dm.update_last_seen(id);
                dm.update_online();
                None
            }
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
