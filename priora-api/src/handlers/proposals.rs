use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{ensure_profile, get_user_by_id, is_admin};
use crate::db::{fetch_category, fetch_namespace_by_slug, fetch_user_public, sort_proposals_by_score};
use crate::membership::{can_create_in_space, get_membership};
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession, OptionalAuthSession};
use crate::models::{
    encode_image_urls, CreateProposalRequest, Namespace, Proposal, ProposalDetail, ProposalEvent,
    ProposalListItem, RankingInsight, TimelineEvent, UpdateProposalRequest, UpdateStatusRequest,
    UpdateTrackerRequest,
};
use crate::ranking::compute_ranking_stats;

async fn record_event(
    pool: &sqlx::SqlitePool,
    proposal_id: &str,
    event_type: &str,
    actor_id: Option<&str>,
    from_value: Option<&str>,
    to_value: Option<&str>,
) -> AppResult<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO proposal_events (id, proposal_id, event_type, actor_id, from_value, to_value, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(proposal_id)
    .bind(event_type)
    .bind(actor_id)
    .bind(from_value)
    .bind(to_value)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_timeline(
    pool: &sqlx::SqlitePool,
    proposal_id: &str,
) -> AppResult<Vec<TimelineEvent>> {
    let rows = sqlx::query_as::<_, ProposalEvent>(
        "SELECT * FROM proposal_events WHERE proposal_id = ? ORDER BY created_at ASC, id ASC",
    )
    .bind(proposal_id)
    .fetch_all(pool)
    .await?;

    let mut timeline = Vec::with_capacity(rows.len());
    for row in rows {
        let actor = if let Some(actor_id) = &row.actor_id {
            Some(fetch_user_public(pool, actor_id).await?)
        } else {
            None
        };

        let (from_user, to_user) = if row.event_type == "tracker_changed" {
            let from_user = if let Some(id) = &row.from_value {
                Some(fetch_user_public(pool, id).await?)
            } else {
                None
            };
            let to_user = if let Some(id) = &row.to_value {
                Some(fetch_user_public(pool, id).await?)
            } else {
                None
            };
            (from_user, to_user)
        } else {
            (None, None)
        };

        timeline.push(TimelineEvent {
            id: row.id,
            event_type: row.event_type,
            actor,
            from_value: row.from_value,
            to_value: row.to_value,
            from_user,
            to_user,
            created_at: row.created_at,
        });
    }

    Ok(timeline)
}

fn status_transition_allowed(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        ("activa", "en_analisis")
            | ("activa", "rechazada")
            | ("en_analisis", "activa")
            | ("en_analisis", "rechazada")
            | ("rechazada", "activa")
    )
}

#[derive(Deserialize)]
pub struct NamespacePath {
    pub namespace: String,
}

#[derive(Deserialize)]
pub struct ProposalPath {
    pub namespace: String,
    pub id: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_filter")]
    filter: String,
    category: Option<String>,
}

fn default_filter() -> String {
    "active".into()
}

async fn resolve_namespace(pool: &sqlx::SqlitePool, slug: &str) -> AppResult<Namespace> {
    fetch_namespace_by_slug(pool, slug).await
}

async fn load_proposal_category(
    pool: &sqlx::SqlitePool,
    category_id: Option<&str>,
) -> AppResult<crate::models::Category> {
    let id = category_id.ok_or_else(|| AppError::BadRequest("category required".into()))?;
    fetch_category(pool, id).await
}

pub async fn fetch_proposal_in_namespace(
    pool: &sqlx::SqlitePool,
    namespace_id: &str,
    id: &str,
) -> AppResult<Proposal> {
    sqlx::query_as::<_, Proposal>(
        "SELECT * FROM proposals WHERE id = ? AND namespace_id = ?",
    )
    .bind(id)
    .bind(namespace_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ProposalListItem>>> {
    let ns = resolve_namespace(&state.pool, &ns_path.namespace).await?;

    let statuses: Vec<&str> = if query.filter == "rejected" {
        vec!["rechazada"]
    } else {
        vec!["activa", "en_analisis"]
    };

    let placeholders = statuses
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let mut sql = format!(
        "SELECT * FROM proposals WHERE namespace_id = ? AND status IN ({placeholders})"
    );
    if query.category.is_some() {
        sql.push_str(" AND category_id = ?");
    }
    sql.push_str(" ORDER BY created_at ASC");

    let mut q = sqlx::query_as::<_, Proposal>(&sql);
    q = q.bind(&ns.id);
    for s in &statuses {
        q = q.bind(*s);
    }
    if let Some(category) = &query.category {
        q = q.bind(category);
    }
    let proposals = q.fetch_all(&state.pool).await?;

    let stats_map = if query.filter == "rejected" {
        std::collections::HashMap::new()
    } else {
        compute_ranking_stats(&state.pool, &ns.id).await?
    };

    let mut items = Vec::new();
    for p in proposals {
        let author = fetch_user_public(&state.pool, &p.author_id).await?;
        let tracker = if let Some(tid) = &p.tracker_id {
            Some(fetch_user_public(&state.pool, tid).await?)
        } else {
            None
        };
        let category = load_proposal_category(&state.pool, p.category_id.as_deref()).await?;
        let stats = stats_map.get(&p.id);
        let score = stats.map(|s| s.score).unwrap_or(0);
        let rankers_count = stats.map(|s| s.rankers_count).unwrap_or(0);
        let agreement = stats.and_then(|s| s.agreement.map(str::to_string));
        let image_urls = p.parsed_image_urls();
        items.push(ProposalListItem {
            id: p.id,
            title: p.title,
            description: p.description,
            image_urls,
            status: p.status,
            author,
            tracker,
            category,
            rank_position: 0,
            score,
            rankers_count,
            agreement,
            created_at: p.created_at,
        });
    }

    if query.filter != "rejected" {
        items = sort_proposals_by_score(items);
    }

    Ok(Json(items))
}

fn build_ranking_insight(
    stats: Option<&crate::ranking::ProposalRankingStats>,
    rank_position: Option<i64>,
    active_count: i64,
    your_position: Option<i64>,
    your_points: Option<i64>,
) -> RankingInsight {
    let rankers = stats.map(|s| s.rankers_count).unwrap_or(0);
    let top3 = stats.map(|s| s.top3_count).unwrap_or(0);
    let avg = stats.map(|s| s.avg_position).unwrap_or(0.0);
    let agreement = stats.and_then(|s| s.agreement.map(str::to_string));
    let points_for_first = active_count.max(1);

    let summary = if rankers == 0 {
        "Todavía nadie priorizó esta propuesta.".into()
    } else {
        let mut parts = Vec::new();
        if let Some(pos) = rank_position {
            parts.push(format!("Está #{pos} en el ranking global"));
        }
        parts.push(format!(
            "{rankers} vecino{} la priorizaron",
            if rankers == 1 { "" } else { "s" }
        ));
        if top3 > 0 {
            parts.push(format!(
                "{top3} la pusieron entre sus 3 primeras"
            ));
        }
        if avg > 0.0 {
            parts.push(format!(
                "posición media {:.1}",
                avg + 1.0
            ));
        }
        match agreement.as_deref() {
            Some("consensus") => parts.push("hay consenso claro".into()),
            Some("polarized") => parts.push("divide opiniones".into()),
            _ => {}
        }
        if let (Some(yp), Some(ypts)) = (your_position, your_points) {
            parts.push(format!(
                "tu #{} aporta {ypts} punto{}",
                yp + 1,
                if ypts == 1 { "" } else { "s" }
            ));
        }
        let mut s = parts[0].clone();
        for part in parts.iter().skip(1) {
            s.push_str("; ");
            s.push_str(part);
        }
        s.push('.');
        // Capitalize first letter already handled
        let mut chars = s.chars();
        match chars.next() {
            Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
            None => s,
        }
    };

    RankingInsight {
        rankers_count: rankers,
        top3_count: top3,
        avg_position: avg,
        agreement,
        summary,
        your_position,
        your_points,
        points_for_first,
    }
}

pub async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    auth: OptionalAuthSession,
) -> AppResult<Json<ProposalDetail>> {
    let ns = resolve_namespace(&state.pool, &path.namespace).await?;
    let p = fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    let author = fetch_user_public(&state.pool, &p.author_id).await?;
    let tracker = if let Some(tid) = &p.tracker_id {
        Some(fetch_user_public(&state.pool, tid).await?)
    } else {
        None
    };
    let category = load_proposal_category(&state.pool, p.category_id.as_deref()).await?;

    let stats_map = compute_ranking_stats(&state.pool, &ns.id).await?;
    let stats = stats_map.get(&p.id);
    let score = stats.map(|s| s.score).unwrap_or(0);
    let rankers_count = stats.map(|s| s.rankers_count).unwrap_or(0);
    let agreement = stats.and_then(|s| s.agreement.map(str::to_string));

    let ranked: Vec<ProposalListItem> = list(
        State(state.clone()),
        Path(NamespacePath {
            namespace: path.namespace.clone(),
        }),
        Query(ListQuery {
            filter: "active".into(),
            category: None,
        }),
    )
    .await?
    .0;
    let rank_position = ranked
        .iter()
        .position(|item| item.id == path.id)
        .map(|i| (i + 1) as i64);
    let active_count = ranked.len() as i64;

    let (your_position, your_points) = if let Some(session) = &auth.session {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT position FROM user_rankings
             WHERE user_id = ? AND namespace_id = ? AND proposal_id = ?",
        )
        .bind(&session.user.id)
        .bind(&ns.id)
        .bind(&path.id)
        .fetch_optional(&state.pool)
        .await?;

        let list_len: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_rankings WHERE user_id = ? AND namespace_id = ?",
        )
        .bind(&session.user.id)
        .bind(&ns.id)
        .fetch_one(&state.pool)
        .await?;

        match row {
            Some((pos,)) if list_len > 0 => (Some(pos), Some(list_len - pos)),
            _ => (None, None),
        }
    } else {
        (None, None)
    };

    let ranking_insight = build_ranking_insight(
        stats,
        rank_position,
        active_count,
        your_position,
        your_points,
    );

    let timeline = load_timeline(&state.pool, &path.id).await?;
    let image_urls = p.parsed_image_urls();

    Ok(Json(ProposalDetail {
        id: p.id,
        title: p.title,
        description: p.description,
        image_urls,
        status: p.status,
        author,
        tracker,
        category,
        score,
        rank_position,
        rankers_count,
        agreement,
        ranking_insight,
        timeline,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
    Json(body): Json<CreateProposalRequest>,
) -> AppResult<Json<ProposalDetail>> {
    let ns = resolve_namespace(&state.pool, &ns_path.namespace).await?;
    let user = &session.user;
    let membership = get_membership(&state.pool, &ns.id, &user.id).await?;
    if !can_create_in_space(user, membership.as_ref()) {
        return Err(AppError::Forbidden);
    }
    ensure_profile(user)?;
    if body.title.trim().is_empty() || body.description.trim().is_empty() {
        return Err(AppError::BadRequest("title and description required".into()));
    }
    if body.category_id.trim().is_empty() {
        return Err(AppError::BadRequest("category required".into()));
    }
    fetch_category(&state.pool, body.category_id.trim()).await?;

    let image_urls_json = encode_image_urls(body.image_urls.as_deref().unwrap_or(&[]))
        .map_err(AppError::BadRequest)?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO proposals (id, title, description, image_urls, status, author_id, category_id, namespace_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'activa', ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(body.title.trim())
    .bind(body.description.trim())
    .bind(&image_urls_json)
    .bind(&user.id)
    .bind(body.category_id.trim())
    .bind(&ns.id)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await?;

    record_event(
        &state.pool,
        &id,
        "created",
        Some(&user.id),
        None,
        Some("activa"),
    )
    .await?;

    get_one(
        State(state),
        Path(ProposalPath {
            namespace: ns_path.namespace,
            id,
        }),
        OptionalAuthSession {
            session: Some(session),
        },
    )
    .await
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    session: AuthSession,
    Json(body): Json<UpdateProposalRequest>,
) -> AppResult<Json<ProposalDetail>> {
    let ns = resolve_namespace(&state.pool, &path.namespace).await?;
    let user = &session.user;
    let p = fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    let can_edit = is_admin(user) || (p.author_id == user.id && p.status == "activa");
    if !can_edit {
        return Err(AppError::Forbidden);
    }

    let title = body.title.unwrap_or(p.title);
    let description = body.description.unwrap_or(p.description);
    let image_urls_json = match body.image_urls {
        Some(urls) => encode_image_urls(&urls).map_err(AppError::BadRequest)?,
        None => p.image_urls,
    };
    let category_id = match body.category_id {
        Some(category_id) => {
            fetch_category(&state.pool, category_id.trim()).await?;
            category_id.trim().to_string()
        }
        None => p
            .category_id
            .ok_or_else(|| AppError::BadRequest("category required".into()))?,
    };
    let now = Utc::now();

    sqlx::query(
        "UPDATE proposals SET title = ?, description = ?, image_urls = ?, category_id = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&title)
    .bind(&description)
    .bind(&image_urls_json)
    .bind(&category_id)
    .bind(now)
    .bind(&path.id)
    .execute(&state.pool)
    .await?;

    get_one(
        State(state),
        Path(path),
        OptionalAuthSession {
            session: Some(session),
        },
    )
    .await
}

pub async fn update_status(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    session: AuthSession,
    Json(body): Json<UpdateStatusRequest>,
) -> AppResult<Json<ProposalDetail>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let ns = resolve_namespace(&state.pool, &path.namespace).await?;
    let p = fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    let valid = ["activa", "en_analisis", "rechazada"];
    if !valid.contains(&body.status.as_str()) {
        return Err(AppError::BadRequest("invalid status".into()));
    }
    if !status_transition_allowed(&p.status, &body.status) {
        return Err(AppError::BadRequest(format!(
            "transition from {} to {} is not allowed",
            p.status, body.status
        )));
    }

    if p.status != body.status {
        sqlx::query("UPDATE proposals SET status = ?, updated_at = ? WHERE id = ?")
            .bind(&body.status)
            .bind(Utc::now())
            .bind(&path.id)
            .execute(&state.pool)
            .await?;

        record_event(
            &state.pool,
            &path.id,
            "status_changed",
            Some(&session.user.id),
            Some(&p.status),
            Some(&body.status),
        )
        .await?;
    }

    get_one(
        State(state),
        Path(path),
        OptionalAuthSession {
            session: Some(session),
        },
    )
    .await
}

pub async fn update_tracker(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    session: AuthSession,
    Json(body): Json<UpdateTrackerRequest>,
) -> AppResult<Json<ProposalDetail>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let ns = resolve_namespace(&state.pool, &path.namespace).await?;
    let p = fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    if let Some(tracker_id) = &body.tracker_id {
        get_user_by_id(&state.pool, tracker_id).await?;
    }

    let changed = p.tracker_id != body.tracker_id;
    if changed {
        sqlx::query("UPDATE proposals SET tracker_id = ?, updated_at = ? WHERE id = ?")
            .bind(&body.tracker_id)
            .bind(Utc::now())
            .bind(&path.id)
            .execute(&state.pool)
            .await?;

        record_event(
            &state.pool,
            &path.id,
            "tracker_changed",
            Some(&session.user.id),
            p.tracker_id.as_deref(),
            body.tracker_id.as_deref(),
        )
        .await?;
    }

    get_one(
        State(state),
        Path(path),
        OptionalAuthSession {
            session: Some(session),
        },
    )
    .await
}
