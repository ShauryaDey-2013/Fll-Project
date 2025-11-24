#![allow(clippy::needless_return)]


use rocket::serde::json::Json;

use rocket::serde::{Deserialize, Serialize};
use rocket::fairing::{Fairing, Info, Kind};

use rocket::{

    fairing::{Fairing, Info, Kind},
    http::{Header, Status},

    Request, Response, State,

};


use std::sync::{
    atomic::{AtomicI32, Ordering},
};
    Mutex,

// -----------------------------------------------------------------------------
// CORS Fairing (development-friendly; adjust for production)
// -----------------------------------------------------------------------------

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, DELETE, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// Satisfy preflight requests under the /api mount
#[rocket::options("/<_..>")]
fn options_preflight() {}

// -----------------------------------------------------------------------------
// Data Models (aligned with the frontend in MainApp/frontend/archeology/src/main.rs)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct AnalyzeRequest {
    image_data: String,
    tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct AnalyzeResponse {
    name: String,
    description: String,
    confidence: f32,
    method: String,
    tier: String,
    analysis_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreateArtifactRequest {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    tier: String,
    image_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ApiArtifact {
    id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    tier: String,
    thumbnail: Option<String>,
    image_data: Option<String>,
    uploaded_at: Option<String>,
    analyzed_at: Option<String>,
    confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreateArtifactResponse {
    id: i32,
}

// -----------------------------------------------------------------------------
// In-memory Store (simple placeholder; replace with DB as needed)
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Store {
    artifacts: Mutex<Vec<ApiArtifact>>,
    next_id: AtomicI32,
}

impl Store {
    fn new() -> Self {
        Self {
            artifacts: Mutex::new(Vec::new()),
            next_id: AtomicI32::new(1),
        }
    }

    fn next_id(&self) -> i32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", now.as_secs(), now.subsec_millis())
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

// -----------------------------------------------------------------------------
// Routes (mounted under /api to match the frontend's API_BASE_URL)
// -----------------------------------------------------------------------------

// GET /api
#[rocket::get("/")]
fn health() -> &'static str {
    "OK"
}

// POST /api/analyze
#[rocket::post("/analyze", data = "<payload>")]
async fn analyze(payload: Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let tier = payload.tier.clone();

    // Mock inference characteristics based on tier and payload size
    let base_ms = 250u64;
    let multiplier = match tier.as_str() {
        "fast" => 1,
        "balanced" => 2,
        "thorough" => 4,
        _ => 2,
    };
    let size_factor = (payload.image_data.len() as u64 / 50_000).max(1);
    let elapsed_ms = base_ms * multiplier * size_factor;

    let response = AnalyzeResponse {
        name: "Unclassified Artifact".to_string(),
        description: "An artifact of unknown origin pending human review.".to_string(),
        confidence: match tier.as_str() {
            "fast" => 0.62,
            "balanced" => 0.78,
            "thorough" => 0.91,
            _ => 0.75,
        },
        method: "mock-ai".to_string(),
        tier,
        analysis_time: format!("{}ms", elapsed_ms),
    };

    Json(response)
}

// POST /api/artifacts
#[rocket::post("/artifacts", data = "<payload>")]
async fn create_artifact(
    state: &State<Store>,
    payload: Json<CreateArtifactRequest>,
) -> Json<CreateArtifactResponse> {
    let id = state.next_id();
    let now = now_string();

    let api_artifact = ApiArtifact {
        id,
        name: payload.name.clone(),
        description: payload.description.clone(),
        tags: payload.tags.clone(),
        tier: payload.tier.clone(),
        thumbnail: None, // In real implementation, generate a thumbnail
        image_data: payload.image_data.clone(),
        uploaded_at: Some(now),
        analyzed_at: None,
        confidence: None,
    };

    {
        let mut artifacts = state.artifacts.lock().expect("artifacts mutex poisoned");
        artifacts.push(api_artifact);
    }

    Json(CreateArtifactResponse { id })
}

// GET /api/artifacts
#[rocket::get("/artifacts")]
async fn get_artifacts(state: &State<Store>) -> Json<Vec<ApiArtifact>> {
    let artifacts = state
        .artifacts
        .lock()
        .expect("artifacts mutex poisoned")
        .clone();
    Json(artifacts)
}

// GET /api/artifacts/search?q=term
#[rocket::get("/artifacts/search?<q>")]
async fn search_artifacts(state: &State<Store>, q: Option<&str>) -> Json<Vec<ApiArtifact>> {
    let query = q.unwrap_or("").trim().to_string();

    if query.is_empty() {
        return get_artifacts(state).await;
    }

    let artifacts = state.artifacts.lock().expect("artifacts mutex poisoned");

    let results = artifacts
        .iter()
        .cloned()
        .filter(|a| {
            contains_case_insensitive(&a.name, &query)
                || a.description
                    .as_ref()
                    .map(|d| contains_case_insensitive(d, &query))
                    .unwrap_or(false)
                || a.tags.iter().any(|t| contains_case_insensitive(t, &query))
        })
        .collect::<Vec<_>>();

    Json(results)
}

// DELETE /api/artifacts/<id>
#[rocket::delete("/artifacts/<id>")]
async fn delete_artifact(state: &State<Store>, id: i32) -> Status {
    let mut artifacts = state.artifacts.lock().expect("artifacts mutex poisoned");
    let before = artifacts.len();
    artifacts.retain(|a| a.id != id);
    let after = artifacts.len();

    if after < before {
        Status::NoContent
    } else {
        Status::NotFound
    }
}

// -----------------------------------------------------------------------------
// Rocket builder (exposed for use from main)
// -----------------------------------------------------------------------------

pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let config = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 8000))
        .merge(("cli_colors", false));

    rocket::custom(config)
        .manage(Store::new())
        .attach(Cors)
        .mount(
            "/api",
            rocket::routes![
                options_preflight,
                health,
                analyze,
                create_artifact,
                get_artifacts,
                search_artifacts,
                delete_artifact
            ],
        )
}
