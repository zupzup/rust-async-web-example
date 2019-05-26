use super::data::{
    ActivitiesResponse, ActivityRequest, ActivityResponse, EditActivityRequest, ErrorListResponse,
    SignInResponse,
};
use super::handlers::AnalyzerError;
use failure::Error;
use futures::Future;
use reqwest::r#async::{Client, Response};
use std::collections::HashMap;

const BASE_URL: &str = "https://testing.timeular.com/api/v2";

pub fn get_activities(jwt: &str) -> impl Future<Item = ActivitiesResponse, Error = reqwest::Error> {
    let activities_path = format!("{}/activities", BASE_URL);
    get(&activities_path, jwt).and_then(|mut res: Response| res.json::<ActivitiesResponse>())
}

pub fn get_activity(
    id: &str,
    jwt: &str,
) -> impl Future<Item = ActivityResponse, Error = AnalyzerError> {
    let cid = String::from(id);
    let activities_path = format!("{}/activities", BASE_URL);
    get(&activities_path, jwt)
        .and_then(|mut res: Response| res.json::<ActivitiesResponse>())
        .map_err(|_| AnalyzerError::ExternalServiceError)
        .and_then(move |ar: ActivitiesResponse| {
            let activity = ar
                .activities
                .iter()
                .cloned()
                .find(|activity| activity.id == cid);
            match activity {
                Some(v) => futures::future::ok(v),
                None => futures::future::err(AnalyzerError::ActivityNotFoundError),
            }
        })
}

pub fn create_activity(
    activity: &ActivityRequest,
    jwt: &str,
) -> impl Future<Item = ActivityResponse, Error = Error> {
    let mut body: HashMap<&str, &str> = HashMap::new();
    body.insert("name", &activity.name);
    body.insert("color", &activity.color);
    body.insert("integration", &activity.integration);
    let path = format!("{}/activities", BASE_URL);
    post(&path, &body, jwt)
        .and_then(|mut res: Response| res.json::<ActivityResponse>())
        .map_err(|e| format_err!("error creating activity, reason: {}", e))
}

pub fn edit_activity(
    id: &str,
    activity: &EditActivityRequest,
    jwt: &str,
) -> impl Future<Item = ActivityResponse, Error = Error> {
    let mut body: HashMap<&str, &str> = HashMap::new();
    let act = activity.clone();
    match act.name.as_ref() {
        Some(v) => body.insert("name", v),
        None => None,
    };
    match act.color.as_ref() {
        Some(v) => body.insert("color", v),
        None => None,
    };
    let path = format!("{}/activities/{}", BASE_URL, id);
    patch(&path, &body, jwt)
        .and_then(|mut res: Response| res.json::<ActivityResponse>())
        .map_err(|e| format_err!("error editing activity, reason: {}", e))
}

pub fn delete_activity(
    id: &str,
    jwt: &str,
) -> impl Future<Item = ErrorListResponse, Error = Error> {
    let path = format!("{}/activities/{}", BASE_URL, id);
    delete(&path, jwt)
        .and_then(|mut res: Response| res.json::<ErrorListResponse>())
        .map_err(|e| format_err!("error deleting activity, reason: {}", e))
}

pub fn get_jwt(
    api_key: &str,
    api_secret: &str,
) -> impl Future<Item = SignInResponse, Error = Error> {
    let mut body = HashMap::new();
    body.insert("apiKey", api_key);
    body.insert("apiSecret", api_secret);
    let jwt_path = format!("{}/developer/sign-in", BASE_URL);
    post(&jwt_path, &body, "")
        .and_then(|mut res: Response| res.json::<SignInResponse>())
        .map_err(|e| format_err!("error logging in, reason: {}", e))
}

fn get(path: &str, jwt: &str) -> impl Future<Item = Response, Error = reqwest::Error> {
    let client = Client::new();
    client
        .get(path)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .and_then(|res: Response| futures::future::ok(res))
        .map_err(|err| {
            println!("Error during get request: {}", err);
            err
        })
}

fn post(
    path: &str,
    body: &HashMap<&str, &str>,
    jwt: &str,
) -> impl Future<Item = Response, Error = reqwest::Error> {
    let client = Client::new();
    client
        .post(path)
        .json(&body)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .and_then(|res: Response| futures::future::ok(res))
        .map_err(|err| {
            println!("Error during post request: {}", err);
            err
        })
}

fn patch(
    path: &str,
    body: &HashMap<&str, &str>,
    jwt: &str,
) -> impl Future<Item = Response, Error = reqwest::Error> {
    let client = Client::new();
    client
        .patch(path)
        .json(&body)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .and_then(|res: Response| futures::future::ok(res))
        .map_err(|err| {
            println!("Error during patch request: {}", err);
            err
        })
}

fn delete(path: &str, jwt: &str) -> impl Future<Item = Response, Error = reqwest::Error> {
    let client = Client::new();
    client
        .delete(path)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .and_then(|res: Response| futures::future::ok(res))
        .map_err(|err| {
            println!("Error during delete request: {}", err);
            err
        })
}
