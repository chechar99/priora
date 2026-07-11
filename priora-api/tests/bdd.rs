use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use cucumber::{given, then, when, World};
use http_body_util::BodyExt;
use priora_api::config::Config;
use priora_api::handlers::{build_router, AppState};
use priora_api::ranking::compute_borda_scores;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct BddWorld {
    _tmp: TempDir,
    pool: SqlitePool,
    app: Router,
    namespace_slug: String,
    namespace_id: String,
    proposal_id: String,
    tokens: HashMap<String, String>,
    user_ids: HashMap<String, String>,
    last_status: Option<StatusCode>,
    last_json: Option<Value>,
    last_score: Option<i64>,
}

impl BddWorld {
    async fn new() -> Self {
        let tmp = TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("bdd.db");
        let url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .expect("connect sqlite");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        let config = Config {
            database_url: url,
            jwt_secret: "bdd-test-secret".into(),
            host: "127.0.0.1".into(),
            port: 0,
            frontend_url: "http://localhost:5173".into(),
            dev_auth: true,
            dev_impersonation: false,
            impersonate_query_key: "priora_as".into(),
            seed_demo_data: false,
            google_client_id: None,
            google_client_secret: None,
            google_redirect_uri: None,
        };

        let app = build_router(Arc::new(AppState {
            pool: pool.clone(),
            config,
        }));

        Self {
            _tmp: tmp,
            pool,
            app,
            namespace_slug: String::new(),
            namespace_id: String::new(),
            proposal_id: String::new(),
            tokens: HashMap::new(),
            user_ids: HashMap::new(),
            last_status: None,
            last_json: None,
            last_score: None,
        }
    }

    async fn request(
        &mut self,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Option<Value>) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header("Authorization", format!("Bearer {t}"));
        }
        let req = if let Some(b) = body {
            builder
                .header("content-type", "application/json")
                .body(Body::from(b.to_string()))
                .unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        };

        let response = self.app.clone().oneshot(req).await.expect("response");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            None
        } else {
            serde_json::from_slice(&bytes).ok()
        };
        self.last_status = Some(status);
        self.last_json = json.clone();
        (status, json)
    }

    async fn login_as(&mut self, key: &str, email: &str, name: &str, role: &str) {
        let (_, json) = self
            .request(
                "POST",
                "/api/auth/dev-login",
                None,
                Some(json!({
                    "email": email,
                    "name": name,
                    "role": role,
                })),
            )
            .await;
        let json = json.expect("login json");
        let token = json["token"].as_str().expect("token").to_string();
        let user_id = json["user"]["id"].as_str().expect("user id").to_string();

        // Complete profile so membership/comment/ranking gates pass.
        let (_, _) = self
            .request(
                "PATCH",
                "/api/users/me",
                Some(&token),
                Some(json!({
                    "street": "Calle de Prueba 123",
                    "city": "Barrio Test",
                    "floor_apt": null,
                    "postal_code": null,
                })),
            )
            .await;

        self.tokens.insert(key.to_string(), token);
        self.user_ids.insert(key.to_string(), user_id);
    }

    fn token(&self, key: &str) -> String {
        self.tokens
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("missing token for {key}"))
    }

    fn user_id(&self, key: &str) -> String {
        self.user_ids
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("missing user id for {key}"))
    }

    async fn proposal_score(&self) -> i64 {
        let scores = compute_borda_scores(&self.pool, &self.namespace_id)
            .await
            .expect("scores");
        *scores.get(&self.proposal_id).unwrap_or(&0)
    }
}

#[given(regex = r#"^un espacio "([^"]+)" con datos mínimos$"#)]
async fn espacio_minimo(world: &mut BddWorld, slug: String) {
    let ns = sqlx::query_as::<_, (String, String)>(
        "SELECT id, slug FROM namespaces WHERE slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&world.pool)
    .await
    .expect("query namespace");

    let ns_id = if let Some((id, _)) = ns {
        sqlx::query("UPDATE namespaces SET require_member_approval = 0 WHERE id = ?")
            .bind(&id)
            .execute(&world.pool)
            .await
            .expect("reset approval flag");
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO namespaces (id, slug, name, require_member_approval)
             VALUES (?, ?, ?, 0)",
        )
        .bind(&id)
        .bind(&slug)
        .bind(format!("Espacio {slug}"))
        .execute(&world.pool)
        .await
        .expect("insert namespace");
        id
    };

    // Author for the proposal (proponent).
    world
        .login_as(
            "author",
            "autor@priora.test",
            "Autor Proponente",
            "proponent",
        )
        .await;
    let author_id = world.user_id("author");

    let proposal_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO proposals
         (id, title, description, status, author_id, category_id, namespace_id, created_at, updated_at)
         VALUES (?, 'Propuesta BDD', 'Descripción de prueba', 'activa', ?, 'servicios', ?, datetime('now'), datetime('now'))",
    )
    .bind(&proposal_id)
    .bind(&author_id)
    .bind(&ns_id)
    .execute(&world.pool)
    .await
    .expect("insert proposal");

    world.namespace_slug = slug;
    world.namespace_id = ns_id;
    world.proposal_id = proposal_id;
}

#[given("un administrador de plataforma autenticado")]
async fn admin_autenticado(world: &mut BddWorld) {
    world
        .login_as("admin", "admin@priora.test", "Admin Plataforma", "admin")
        .await;
}

#[given("un usuario regular autenticado con perfil completo")]
async fn regular_autenticado(world: &mut BddWorld) {
    world
        .login_as(
            "regular",
            "vecino@priora.test",
            "Vecino Regular",
            "regular",
        )
        .await;
}

#[given("un admin de espacio autenticado")]
async fn space_admin_autenticado(world: &mut BddWorld) {
    world
        .login_as(
            "space_admin",
            "spaceadmin@priora.test",
            "Admin Espacio",
            "regular",
        )
        .await;
    let user_id = world.user_id("space_admin");
    let admin_token = world.token("admin");
    let path = format!("/api/{}/members/{}", world.namespace_slug, user_id);
    let (status, _) = world
        .request(
            "PATCH",
            &path,
            Some(&admin_token),
            Some(json!({ "status": "active", "role": "space_admin" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "failed to assign space_admin");
}

#[given("que el espacio no requiere aprobación de usuarios")]
async fn sin_aprobacion(world: &mut BddWorld) {
    set_approval(world, false).await;
}

#[given("que el espacio requiere aprobación de usuarios")]
async fn con_aprobacion(world: &mut BddWorld) {
    set_approval(world, true).await;
}

async fn set_approval(world: &mut BddWorld, require: bool) {
    let token = world.token("admin");
    let path = format!("/api/namespaces/{}", world.namespace_slug);
    let (status, json) = world
        .request(
            "PATCH",
            &path,
            Some(&token),
            Some(json!({ "require_member_approval": require })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "update namespace failed: {json:?}");
    assert_eq!(
        json.as_ref()
            .and_then(|j| j["require_member_approval"].as_bool()),
        Some(require)
    );
}

#[when("el usuario regular consulta su membresía")]
async fn consulta_membresia(world: &mut BddWorld) {
    consulta_membresia_de(world, "regular").await;
}

#[when("el administrador consulta su membresía")]
async fn admin_consulta_membresia(world: &mut BddWorld) {
    consulta_membresia_de(world, "admin").await;
}

#[when("el admin de espacio consulta su membresía")]
async fn space_admin_consulta_membresia(world: &mut BddWorld) {
    consulta_membresia_de(world, "space_admin").await;
}

async fn consulta_membresia_de(world: &mut BddWorld, who: &str) {
    let token = world.token(who);
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (status, _) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "membership/me failed for {who}");
}

#[when("el administrador activa la aprobación de usuarios")]
async fn admin_activa_aprobacion(world: &mut BddWorld) {
    set_approval(world, true).await;
}

#[when("el administrador desactiva la aprobación de usuarios")]
async fn admin_desactiva_aprobacion(world: &mut BddWorld) {
    set_approval(world, false).await;
}

#[when("el administrador lista los miembros pendientes")]
async fn admin_lista_pendientes(world: &mut BddWorld) {
    lista_miembros(world, "admin", Some("pending")).await;
}

#[when("el usuario regular lista los miembros")]
async fn regular_lista_miembros(world: &mut BddWorld) {
    lista_miembros(world, "regular", None).await;
}

#[when("el admin de espacio lista los miembros")]
async fn space_admin_lista_miembros(world: &mut BddWorld) {
    lista_miembros(world, "space_admin", None).await;
}

async fn lista_miembros(world: &mut BddWorld, who: &str, status: Option<&str>) {
    let token = world.token(who);
    let path = if let Some(s) = status {
        format!("/api/{}/members?status={s}", world.namespace_slug)
    } else {
        format!("/api/{}/members", world.namespace_slug)
    };
    world.request("GET", &path, Some(&token), None).await;
}

#[when("el usuario regular solicita autorización")]
async fn solicita_autorizacion(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/request", world.namespace_slug);
    let (status, json) = world.request("POST", &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "request failed: {json:?}");
}

#[when("el usuario regular publica un comentario")]
async fn publica_comentario(world: &mut BddWorld) {
    comentar(world, "regular").await;
}

#[when("el usuario regular intenta publicar un comentario")]
async fn intenta_comentar(world: &mut BddWorld) {
    comentar(world, "regular").await;
}

async fn comentar(world: &mut BddWorld, who: &str) {
    let token = world.token(who);
    let path = format!(
        "/api/{}/proposals/{}/comments",
        world.namespace_slug, world.proposal_id
    );
    world
        .request(
            "POST",
            &path,
            Some(&token),
            Some(json!({ "content": "Comentario de prueba BDD" })),
        )
        .await;
}

#[when("el usuario regular guarda su priorización")]
async fn guarda_priorizacion(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/rankings/me", world.namespace_slug);
    world
        .request(
            "PUT",
            &path,
            Some(&token),
            Some(json!({ "proposal_ids": [world.proposal_id] })),
        )
        .await;
}

#[when("el administrador aprueba al usuario regular")]
async fn admin_aprueba(world: &mut BddWorld) {
    aprobar(world, "admin", "active").await;
}

#[when("el administrador rechaza al usuario regular")]
async fn admin_rechaza(world: &mut BddWorld) {
    aprobar(world, "admin", "rejected").await;
}

#[when("el admin de espacio aprueba al usuario regular")]
async fn space_admin_aprueba(world: &mut BddWorld) {
    aprobar(world, "space_admin", "active").await;
}

async fn aprobar(world: &mut BddWorld, actor: &str, status: &str) {
    let token = world.token(actor);
    let user_id = world.user_id("regular");
    let path = format!("/api/{}/members/{}", world.namespace_slug, user_id);
    let (code, json) = world
        .request(
            "PATCH",
            &path,
            Some(&token),
            Some(json!({ "status": status })),
        )
        .await;
    assert_eq!(code, StatusCode::OK, "approve/reject failed: {json:?}");
}

#[then("puede comentar en el espacio")]
async fn puede_comentar(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (_, json) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(
        json.as_ref().and_then(|j| j["can_comment"].as_bool()),
        Some(true),
        "expected can_comment=true, got {json:?}"
    );
}

#[then("puede administrar el espacio")]
async fn puede_administrar(world: &mut BddWorld) {
    assert_eq!(
        world
            .last_json
            .as_ref()
            .and_then(|j| j["can_manage_space"].as_bool()),
        Some(true),
        "expected can_manage_space=true, got {:?}",
        world.last_json
    );
}

#[then("no puede administrar el espacio")]
async fn no_puede_administrar(world: &mut BddWorld) {
    assert_eq!(
        world
            .last_json
            .as_ref()
            .and_then(|j| j["can_manage_space"].as_bool()),
        Some(false),
        "expected can_manage_space=false, got {:?}",
        world.last_json
    );
}

#[then("el espacio requiere aprobación de usuarios")]
async fn espacio_requiere_aprobacion(world: &mut BddWorld) {
    assert_eq!(
        world
            .last_json
            .as_ref()
            .and_then(|j| j["require_member_approval"].as_bool()),
        Some(true),
        "expected require_member_approval=true, got {:?}",
        world.last_json
    );
}

#[then("el espacio no requiere aprobación de usuarios")]
async fn espacio_no_requiere_aprobacion(world: &mut BddWorld) {
    assert_eq!(
        world
            .last_json
            .as_ref()
            .and_then(|j| j["require_member_approval"].as_bool()),
        Some(false),
        "expected require_member_approval=false, got {:?}",
        world.last_json
    );
}

#[then("la lista de miembros incluye al usuario regular")]
async fn lista_incluye_regular(world: &mut BddWorld) {
    let user_id = world.user_id("regular");
    let list = world
        .last_json
        .as_ref()
        .and_then(|j| j.as_array())
        .expect("expected members array");
    assert!(
        list.iter().any(|m| m["user_id"].as_str() == Some(user_id.as_str())),
        "expected user {user_id} in members list, got {list:?}"
    );
}

#[then("no puede comentar en el espacio")]
async fn no_puede_comentar(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (_, json) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(
        json.as_ref().and_then(|j| j["can_comment"].as_bool()),
        Some(false),
        "expected can_comment=false, got {json:?}"
    );
}

#[then("su priorización cuenta en el ranking")]
async fn ranking_cuenta(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (_, json) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(
        json.as_ref().and_then(|j| j["ranking_counts"].as_bool()),
        Some(true),
        "expected ranking_counts=true, got {json:?}"
    );
}

#[then("su priorización no cuenta en el ranking")]
async fn ranking_no_cuenta(world: &mut BddWorld) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (_, json) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(
        json.as_ref().and_then(|j| j["ranking_counts"].as_bool()),
        Some(false),
        "expected ranking_counts=false, got {json:?}"
    );
}

#[then(regex = r#"^su membresía queda en estado "([^"]+)"$"#)]
async fn membresia_estado(world: &mut BddWorld, status: String) {
    let token = world.token("regular");
    let path = format!("/api/{}/membership/me", world.namespace_slug);
    let (_, json) = world.request("GET", &path, Some(&token), None).await;
    assert_eq!(
        json.as_ref()
            .and_then(|j| j["membership"]["status"].as_str()),
        Some(status.as_str()),
        "expected membership status {status}, got {json:?}"
    );
}

#[then("la respuesta es exitosa")]
async fn respuesta_ok(world: &mut BddWorld) {
    let status = world.last_status.expect("no status");
    assert!(
        status.is_success(),
        "expected success, got {status:?} body={:?}",
        world.last_json
    );
}

#[then("la respuesta es prohibida")]
async fn respuesta_forbidden(world: &mut BddWorld) {
    assert_eq!(
        world.last_status,
        Some(StatusCode::FORBIDDEN),
        "expected 403, got {:?} body={:?}",
        world.last_status,
        world.last_json
    );
}

#[then("el score de la propuesta refleja su priorización")]
async fn score_refleja(world: &mut BddWorld) {
    let score = world.proposal_score().await;
    world.last_score = Some(score);
    assert!(
        score > 0,
        "expected proposal score > 0 after counting ranking, got {score}"
    );
}

#[then("el score de la propuesta no refleja su priorización")]
async fn score_no_refleja(world: &mut BddWorld) {
    let score = world.proposal_score().await;
    world.last_score = Some(score);
    assert_eq!(
        score, 0,
        "expected proposal score 0 while pending, got {score}"
    );
}

#[tokio::main]
async fn main() {
    BddWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
