use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;

use crate::db::fetch_namespace_by_slug;
use crate::error::{AppError, AppResult};
use crate::handlers::proposals::NamespacePath;
use crate::handlers::{AppState, AuthSession};
use crate::membership::{
    can_manage_space, get_membership, MEMBER_STATUS_ACTIVE, MEMBER_STATUS_PENDING,
};
use crate::models::{DashboardProposalSummary, SpaceDashboard};
use crate::ranking::{compute_ranking_stats, count_eligible_rankers};

pub async fn dashboard(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<SpaceDashboard>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;
    let membership = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, membership.as_ref()) {
        return Err(AppError::Forbidden);
    }

    let active_members: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM namespace_members WHERE namespace_id = ? AND status = ?",
    )
    .bind(&ns.id)
    .bind(MEMBER_STATUS_ACTIVE)
    .fetch_one(&state.pool)
    .await?;

    let pending_approvals: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM namespace_members WHERE namespace_id = ? AND status = ?",
    )
    .bind(&ns.id)
    .bind(MEMBER_STATUS_PENDING)
    .fetch_one(&state.pool)
    .await?;

    let active_proposals: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM proposals
         WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')",
    )
    .bind(&ns.id)
    .fetch_one(&state.pool)
    .await?;

    let members_who_prioritized = count_eligible_rankers(&state.pool, &ns.id).await?;

    let active_who_ranked: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT ur.user_id) FROM user_rankings ur
         INNER JOIN namespace_members nm
           ON nm.user_id = ur.user_id AND nm.namespace_id = ur.namespace_id
         WHERE ur.namespace_id = ? AND nm.status = ?",
    )
    .bind(&ns.id)
    .bind(MEMBER_STATUS_ACTIVE)
    .fetch_one(&state.pool)
    .await?;

    let prioritization_pct = if active_members > 0 {
        Some(((active_who_ranked as f64) / (active_members as f64) * 1000.0).round() / 10.0)
    } else if members_who_prioritized > 0 {
        // Espacio abierto sin filas de membresía: quienes priorizaron son el 100% del grupo activo.
        Some(100.0)
    } else {
        Some(0.0)
    };

    let stats_map = compute_ranking_stats(&state.pool, &ns.id).await?;

    let proposals: Vec<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, title, created_at FROM proposals
         WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')
         ORDER BY created_at ASC",
    )
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    let mut scored: Vec<(DashboardProposalSummary, f64, i64)> = Vec::new();
    for (id, title, created_at) in &proposals {
        let stats = stats_map.get(id);
        let score = stats.map(|s| s.score).unwrap_or(0);
        let rankers = stats.map(|s| s.rankers_count).unwrap_or(0);
        let agreement = stats.and_then(|s| s.agreement.map(str::to_string));
        let stddev = stats.map(|s| s.relative_stddev).unwrap_or(0.0);
        scored.push((
            DashboardProposalSummary {
                id: id.clone(),
                title: title.clone(),
                rank_position: 0,
                score,
                rankers_count: rankers,
                agreement,
            },
            stddev,
            created_at.timestamp(),
        ));
    }

    scored.sort_by(|a, b| b.0.score.cmp(&a.0.score).then_with(|| a.2.cmp(&b.2)));
    for (i, item) in scored.iter_mut().enumerate() {
        item.0.rank_position = (i + 1) as i64;
    }

    let mut consensual: Vec<_> = scored
        .iter()
        .filter(|(p, _, _)| p.agreement.as_deref() == Some("consensus"))
        .cloned()
        .collect();
    consensual.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let most_consensual: Vec<_> = consensual
        .into_iter()
        .take(5)
        .map(|(p, _, _)| p)
        .collect();

    let mut polarized: Vec<_> = scored
        .iter()
        .filter(|(p, _, _)| p.agreement.as_deref() == Some("polarized"))
        .cloned()
        .collect();
    polarized.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let most_polarized: Vec<_> = polarized
        .into_iter()
        .take(5)
        .map(|(p, _, _)| p)
        .collect();

    Ok(Json(SpaceDashboard {
        require_member_approval: ns.require_member_approval,
        active_members,
        members_who_prioritized,
        prioritization_pct,
        pending_approvals,
        active_proposals,
        most_consensual,
        most_polarized,
    }))
}
