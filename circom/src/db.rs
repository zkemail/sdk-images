use anyhow::Result;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};

pub async fn update_verifier_contract_address(
    pool: &Pool<Postgres>,
    id: &str,
    address: &str,
) -> Result<()> {
    let uuid_id = Uuid::parse_str(id)?;
    let query = r#"
        UPDATE blueprints
        SET verifier_contract_address = $1
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(address)
        .bind(uuid_id)
        .execute(pool)
        .await?;

    Ok(())
}

// pub async fn update_ptau(pool: &Pool<Postgres>, id: Uuid, ptau: usize) -> Result<()> {
//     let query = r#"
//         UPDATE blueprints
//         SET ptau = $1
//         WHERE id = $2
//     "#;

//     sqlx::query(query)
//         .bind(ptau as i32)
//         .bind(id)
//         .execute(pool)
//         .await?;

//     Ok(())
// }
