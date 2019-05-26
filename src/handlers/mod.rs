use super::data::{ActivityRequest, EditActivityRequest};
use super::external;
use super::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::{error, HttpResponse, Responder};
use failure::Fail;
use futures::future::Future;

#[derive(Fail, Debug)]
pub enum AnalyzerError {
    #[fail(display = "External Service Error")]
    ExternalServiceError,
    #[fail(display = "Activity Not Found Error")]
    ActivityNotFoundError,
}

impl error::ResponseError for AnalyzerError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AnalyzerError::ExternalServiceError => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("external service error"),
            AnalyzerError::ActivityNotFoundError => HttpResponse::NotFound()
                .content_type("text/plain")
                .body("activity not found"),
        }
    }
    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}

pub fn health() -> impl Responder {
    "OK".to_string()
}

pub fn get_activities(
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = AnalyzerError> {
    let jwt = &data.jwt;
    let log = data.log.clone();
    external::get_activities(jwt)
        .map_err(move |e| {
            error!(log, "Get Activities ExternalServiceError: {}", e);
            AnalyzerError::ExternalServiceError
        })
        .and_then(|res| json_ok(&res))
}

pub fn get_activity(
    data: Data<AppState>,
    activity_id: Path<String>,
) -> impl Future<Item = HttpResponse, Error = AnalyzerError> {
    let jwt = &data.jwt;
    let log = data.log.clone();
    external::get_activity(&activity_id, jwt)
        .map_err(move |e| {
            error!(log, "Get Activity Error: {}", e);
            e
        })
        .and_then(|res| json_ok(&res))
}

pub fn create_activity(
    data: Data<AppState>,
    activity: Json<ActivityRequest>,
) -> impl Future<Item = HttpResponse, Error = AnalyzerError> {
    let jwt = &data.jwt;
    let log = data.log.clone();
    info!(log, "creating activity {:?}", activity);
    external::create_activity(&activity, jwt)
        .map_err(move |e| {
            error!(log, "Create Activity ExternalServiceError {}", e);
            AnalyzerError::ExternalServiceError
        })
        .and_then(|res| json_ok(&res))
}

pub fn edit_activity(
    data: Data<AppState>,
    activity: Json<EditActivityRequest>,
    activity_id: Path<String>,
) -> impl Future<Item = HttpResponse, Error = AnalyzerError> {
    let jwt = &data.jwt;
    let log = data.log.clone();
    info!(log, "editing activity {:?}", activity);
    external::edit_activity(&activity_id, &activity, jwt)
        .map_err(move |e| {
            error!(log, "Edit Activity ExternalServiceError {}", e);
            AnalyzerError::ExternalServiceError
        })
        .and_then(|res| json_ok(&res))
}

pub fn delete_activity(
    data: Data<AppState>,
    activity_id: Path<String>,
) -> impl Future<Item = HttpResponse, Error = AnalyzerError> {
    let jwt = &data.jwt;
    let log = data.log.clone();
    info!(log, "deleting activity {}", activity_id);
    external::delete_activity(&activity_id, jwt)
        .map_err(move |e| {
            error!(log, "Delete Activity ExternalServiceError {}", e);
            AnalyzerError::ExternalServiceError
        })
        .and_then(|res| json_ok(&res))
}

fn json_ok<T: ?Sized>(data: &T) -> Result<HttpResponse, AnalyzerError>
where
    T: serde::ser::Serialize,
{
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&data).unwrap())
        .into())
}
