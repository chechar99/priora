use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub google_sub: String,
    pub email: String,
    pub name: String,
    pub picture_url: Option<String>,
    pub role: String,
    pub street: Option<String>,
    pub floor_apt: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub profile_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub name: String,
    pub picture_url: Option<String>,
    pub role: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            picture_url: u.picture_url,
            role: u.role,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Namespace {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub require_member_approval: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NamespaceMember {
    pub namespace_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub requested_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MembershipMeResponse {
    pub require_member_approval: bool,
    pub membership: Option<NamespaceMemberPublic>,
    pub can_comment: bool,
    pub ranking_counts: bool,
    pub can_manage_space: bool,
    pub can_create_proposal: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamespaceMemberPublic {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub picture_url: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub role: String,
    pub status: String,
    pub requested_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNamespaceRequest {
    pub require_member_approval: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub status: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub status: String,
    pub author_id: String,
    pub tracker_id: Option<String>,
    pub category_id: Option<String>,
    pub namespace_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalListItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub status: String,
    pub author: UserPublic,
    pub tracker: Option<UserPublic>,
    pub category: Category,
    pub rank_position: i64,
    pub score: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalDetail {
    pub id: String,
    pub title: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub status: String,
    pub author: UserPublic,
    pub tracker: Option<UserPublic>,
    pub category: Category,
    pub score: i64,
    pub rank_position: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: String,
    pub proposal_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommentWithAuthor {
    #[serde(flatten)]
    pub comment: Comment,
    pub author: UserPublic,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub street: String,
    pub floor_apt: Option<String>,
    pub city: String,
    pub postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNamespaceRequest {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub category_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProposalRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackerRequest {
    pub tracker_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveRankingRequest {
    pub proposal_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct DevLoginRequest {
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonator: Option<UserPublic>,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    #[serde(flatten)]
    pub user: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonator: Option<UserPublic>,
}

#[derive(Debug, Serialize)]
pub struct RankingResponse {
    pub proposal_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CommentsPage {
    pub comments: Vec<CommentWithAuthor>,
    pub total: i64,
}
