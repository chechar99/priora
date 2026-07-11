use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::{Category, Namespace, ProposalListItem, User, UserPublic};

pub async fn fetch_user_public(pool: &SqlitePool, id: &str) -> AppResult<UserPublic> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(user.into())
}

pub async fn fetch_namespace_by_slug(pool: &SqlitePool, slug: &str) -> AppResult<Namespace> {
    sqlx::query_as::<_, Namespace>("SELECT * FROM namespaces WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or(crate::error::AppError::NotFound)
}

pub async fn fetch_category(pool: &SqlitePool, id: &str) -> AppResult<Category> {
    sqlx::query_as::<_, Category>("SELECT id, name FROM categories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::AppError::BadRequest("invalid category".into()))
}

struct DemoUser {
    id: &'static str,
    email: &'static str,
    name: &'static str,
    role: &'static str,
    street: &'static str,
    city: &'static str,
}

struct DemoProposal {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    status: &'static str,
    author_email: &'static str,
    tracker_email: Option<&'static str>,
    category_id: &'static str,
    namespace_slug: &'static str,
}

pub async fn seed_demo_data(pool: &SqlitePool) -> AppResult<()> {
    let now = Utc::now();

    let users = [
        DemoUser {
            id: "11111111-1111-4111-a111-111111111101",
            email: "admin@priora.local",
            name: "Administrador",
            role: "admin",
            street: "Calle Principal 1",
            city: "Barrio Centro",
        },
        DemoUser {
            id: "11111111-1111-4111-a111-111111111102",
            email: "proponente@priora.local",
            name: "María Proponente",
            role: "proponent",
            street: "Av. Libertad 200",
            city: "Barrio Centro",
        },
        DemoUser {
            id: "11111111-1111-4111-a111-111111111103",
            email: "carlos.mendez@priora.local",
            name: "Carlos Méndez",
            role: "regular",
            street: "Calle Florida 450",
            city: "Barrio Norte",
        },
        DemoUser {
            id: "11111111-1111-4111-a111-111111111104",
            email: "ana.rios@priora.local",
            name: "Ana Ríos",
            role: "regular",
            street: "Pasaje Las Rosas 12",
            city: "Barrio Sur",
        },
        DemoUser {
            id: "11111111-1111-4111-a111-111111111105",
            email: "luis.torres@priora.local",
            name: "Luis Torres",
            role: "regular",
            street: "Av. del Barrio 890",
            city: "Barrio Centro",
        },
        DemoUser {
            id: "11111111-1111-4111-a111-111111111106",
            email: "sofia.navarro@priora.local",
            name: "Sofía Navarro",
            role: "proponent",
            street: "Calle Mitre 77",
            city: "Barrio Oeste",
        },
    ];

    for u in users {
        sqlx::query(
            "INSERT INTO users (id, google_sub, email, name, role, street, city, profile_complete, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
             ON CONFLICT(google_sub) DO UPDATE SET
               name = excluded.name,
               role = excluded.role,
               street = excluded.street,
               city = excluded.city,
               profile_complete = 1,
               updated_at = excluded.updated_at",
        )
        .bind(u.id)
        .bind(format!("dev:{email}", email = u.email))
        .bind(u.email)
        .bind(u.name)
        .bind(u.role)
        .bind(u.street)
        .bind(u.city)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    let _admin_id = user_id_by_email(pool, "admin@priora.local").await?;
    let _maria_id = user_id_by_email(pool, "proponente@priora.local").await?;
    let _carlos_id = user_id_by_email(pool, "carlos.mendez@priora.local").await?;
    let _ana_id = user_id_by_email(pool, "ana.rios@priora.local").await?;
    let _luis_id = user_id_by_email(pool, "luis.torres@priora.local").await?;
    let _sofia_id = user_id_by_email(pool, "sofia.navarro@priora.local").await?;

    let proposals = [
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222201",
            title: "Plaza renovada en el parque central",
            description: "Propuesta para renovar la plaza del parque central con nuevo mobiliario urbano, iluminación LED y áreas verdes. Incluye bancos accesibles y juegos infantiles.",
            status: "activa",
            author_email: "proponente@priora.local",
            tracker_email: Some("admin@priora.local"),
            category_id: "recreacion",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222202",
            title: "Ciclovía en Av. del Barrio",
            description: "Construcción de una ciclovía segregada en Av. del Barrio para mejorar la movilidad segura de ciclistas y peatones.",
            status: "en_analisis",
            author_email: "proponente@priora.local",
            tracker_email: Some("admin@priora.local"),
            category_id: "movilidad",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222203",
            title: "Contenedores de reciclaje",
            description: "Instalación de contenedores de reciclaje separado en cada cuadra del barrio para mejorar la gestión de residuos.",
            status: "activa",
            author_email: "sofia.navarro@priora.local",
            tracker_email: None,
            category_id: "servicios",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222204",
            title: "Ampliación horario biblioteca",
            description: "Extender el horario de la biblioteca vecinal los fines de semana para fomentar la lectura y el estudio.",
            status: "rechazada",
            author_email: "proponente@priora.local",
            tracker_email: None,
            category_id: "servicios",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222205",
            title: "Pintura de murales comunitarios",
            description: "Invitar a artistas locales a pintar murales en los muros del pasaje peatonal, con participación de vecinos en el diseño.",
            status: "activa",
            author_email: "sofia.navarro@priora.local",
            tracker_email: None,
            category_id: "convivencia",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222206",
            title: "Huerta urbana en la plaza lateral",
            description: "Crear una huerta comunitaria con talleres mensuales de jardinería urbana y compostaje.",
            status: "activa",
            author_email: "proponente@priora.local",
            tracker_email: None,
            category_id: "recreacion",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222207",
            title: "Señalización de calles",
            description: "Renovar la señalización vial y peatonal del barrio, incluyendo rampas accesibles en las esquinas.",
            status: "en_analisis",
            author_email: "sofia.navarro@priora.local",
            tracker_email: Some("admin@priora.local"),
            category_id: "transito",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "22222222-2222-4222-a222-222222222208",
            title: "Feria de emprendedores locales",
            description: "Organizar una feria mensual en la plaza para emprendedores del barrio con stands gratuitos.",
            status: "activa",
            author_email: "proponente@priora.local",
            tracker_email: None,
            category_id: "convivencia",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "33333333-3333-4333-a333-333333333301",
            title: "Alumbrado LED en Av. del Norte",
            description: "Reemplazar luminarias antiguas por LED de bajo consumo en Av. del Norte para mejorar la seguridad nocturna.",
            status: "activa",
            author_email: "carlos.mendez@priora.local",
            tracker_email: Some("admin@priora.local"),
            category_id: "seguridad",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "33333333-3333-4333-a333-333333333302",
            title: "Sendero peatonal en el arroyo",
            description: "Construir un sendero peatonal y ciclovía junto al arroyo para conectar el barrio con el parque municipal.",
            status: "activa",
            author_email: "ana.rios@priora.local",
            tracker_email: None,
            category_id: "movilidad",
            namespace_slug: "barrio-test",
        },
        DemoProposal {
            id: "33333333-3333-4333-a333-333333333303",
            title: "Punto verde de reciclaje",
            description: "Instalar un punto verde centralizado para reciclaje de vidrio, plástico y cartón con horario extendido.",
            status: "en_analisis",
            author_email: "proponente@priora.local",
            tracker_email: None,
            category_id: "servicios",
            namespace_slug: "barrio-test",
        },
    ];

    for p in proposals {
        let author_id = user_id_by_email(pool, p.author_email).await?;
        let tracker_id = match p.tracker_email {
            Some(email) => Some(user_id_by_email(pool, email).await?),
            None => None,
        };
        let ns = fetch_namespace_by_slug(pool, p.namespace_slug).await?;
        sqlx::query(
            "INSERT INTO proposals (id, title, description, status, author_id, tracker_id, category_id, namespace_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               description = excluded.description,
               status = excluded.status,
               author_id = excluded.author_id,
               tracker_id = excluded.tracker_id,
               category_id = excluded.category_id,
               namespace_id = excluded.namespace_id,
               updated_at = excluded.updated_at",
        )
        .bind(p.id)
        .bind(p.title)
        .bind(p.description)
        .bind(p.status)
        .bind(&author_id)
        .bind(&tracker_id)
        .bind(p.category_id)
        .bind(&ns.id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    let carlos_id = user_id_by_email(pool, "carlos.mendez@priora.local").await?;
    let ana_id = user_id_by_email(pool, "ana.rios@priora.local").await?;
    let luis_id = user_id_by_email(pool, "luis.torres@priora.local").await?;

    let comments = [
        (
            "comment-001",
            "22222222-2222-4222-a222-222222222201",
            carlos_id.as_str(),
            "Me encanta esta idea. El parque necesita una renovación urgente.",
            5,
        ),
        (
            "comment-002",
            "22222222-2222-4222-a222-222222222201",
            ana_id.as_str(),
            "¿Se contempla iluminación para la noche? Sería importante para la seguridad.",
            4,
        ),
        (
            "comment-003",
            "22222222-2222-4222-a222-222222222202",
            luis_id.as_str(),
            "Apoyo total. Hay muchos ciclistas en Av. del Barrio y hoy es peligroso.",
            3,
        ),
        (
            "comment-004",
            "22222222-2222-4222-a222-222222222203",
            ana_id.as_str(),
            "Ya hay contenedores en algunas cuadras, habría que coordinar con el municipio.",
            2,
        ),
        (
            "comment-005",
            "22222222-2222-4222-a222-222222222205",
            carlos_id.as_str(),
            "Podríamos hacer un taller con los chicos del colegio del barrio.",
            1,
        ),
        (
            "comment-006",
            "22222222-2222-4222-a222-222222222206",
            luis_id.as_str(),
            "Tengo experiencia en huertas, puedo ayudar como voluntario.",
            0,
        ),
    ];

    for (id, proposal_id, author_id, content, days_ago) in comments {
        let created = now - Duration::days(days_ago);
        sqlx::query(
            "INSERT INTO comments (id, proposal_id, author_id, content, created_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(id) DO NOTHING",
        )
        .bind(id)
        .bind(proposal_id)
        .bind(author_id)
        .bind(content)
        .bind(created)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO namespaces (id, slug, name, require_member_approval)
         VALUES ('ns-barrio-test', 'barrio-test', 'Barrio Test', 0)
         ON CONFLICT(slug) DO UPDATE SET name = excluded.name",
    )
    .execute(pool)
    .await?;

    let test_ns = fetch_namespace_by_slug(pool, "barrio-test").await?;

    let rankings: [(&str, &str, &[&str]); 3] = [
        (
            carlos_id.as_str(),
            test_ns.id.as_str(),
            &[
                "22222222-2222-4222-a222-222222222201",
                "22222222-2222-4222-a222-222222222206",
                "22222222-2222-4222-a222-222222222202",
                "22222222-2222-4222-a222-222222222205",
                "22222222-2222-4222-a222-222222222203",
                "22222222-2222-4222-a222-222222222208",
                "22222222-2222-4222-a222-222222222207",
            ],
        ),
        (
            ana_id.as_str(),
            test_ns.id.as_str(),
            &[
                "22222222-2222-4222-a222-222222222202",
                "22222222-2222-4222-a222-222222222201",
                "22222222-2222-4222-a222-222222222203",
                "22222222-2222-4222-a222-222222222208",
                "22222222-2222-4222-a222-222222222206",
                "22222222-2222-4222-a222-222222222205",
                "22222222-2222-4222-a222-222222222207",
            ],
        ),
        (
            luis_id.as_str(),
            test_ns.id.as_str(),
            &[
                "22222222-2222-4222-a222-222222222206",
                "22222222-2222-4222-a222-222222222205",
                "22222222-2222-4222-a222-222222222201",
                "22222222-2222-4222-a222-222222222208",
                "22222222-2222-4222-a222-222222222202",
                "22222222-2222-4222-a222-222222222207",
                "22222222-2222-4222-a222-222222222203",
            ],
        ),
    ];

    for (user_id, namespace_id, proposal_ids) in rankings {
        sqlx::query("DELETE FROM user_rankings WHERE user_id = ? AND namespace_id = ?")
            .bind(user_id)
            .bind(namespace_id)
            .execute(pool)
            .await?;

        for (position, proposal_id) in proposal_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_rankings (user_id, namespace_id, proposal_id, position, updated_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(user_id)
            .bind(namespace_id)
            .bind(*proposal_id)
            .bind(position as i64)
            .bind(now)
            .execute(pool)
            .await?;
        }
    }

    tracing::info!("Demo data seeded (6 users, 11 proposals, barrio-test, comments, rankings)");
    Ok(())
}

async fn user_id_by_email(pool: &SqlitePool, email: &str) -> AppResult<String> {
    let row: (String,) = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub fn sort_proposals_by_score(mut items: Vec<ProposalListItem>) -> Vec<ProposalListItem> {
    items.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
    for (i, item) in items.iter_mut().enumerate() {
        item.rank_position = (i + 1) as i64;
    }
    items
}
