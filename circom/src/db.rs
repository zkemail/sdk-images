use anyhow::Result;
use sqlx::{types::Uuid, Pool, Postgres};

pub async fn update_db(pool: &Pool<Postgres>, id: Uuid, address: &str) -> Result<()> {
    let query = r#"
        UPDATE blueprints
        SET verifier_contract_address = $1
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(address)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
