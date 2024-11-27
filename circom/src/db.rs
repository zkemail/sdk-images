use anyhow::Result;
use sqlx::{types::Uuid, Pool, Postgres};

pub async fn update_db(pool: &Pool<Postgres>, id: Uuid, address: &str, ptau: i32) -> Result<()> {
    let query = r#"
        UPDATE blueprints
        SET verifier_contract_address = $1, ptau = $2
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(address)
        .bind(id)
        .bind(ptau)
        .execute(pool)
        .await?;

    Ok(())
}
