use std::fmt::Error;
use hyper;

pub enum MethodName {
    Login,
    Logout,
    Users,
    Messages,
}
pub fn get_method_name(path: &str) -> Option<MethodName> {
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

// Authorization: Token <sometoken>
pub fn get_token(head: &hyper::http::request::Parts) -> Option<String> {
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
pub fn get_id_from_users_request(path: String) -> Option<usize> {
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
pub fn get_message_request_interval(path: String) -> Result<(usize, usize), Error> {
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
