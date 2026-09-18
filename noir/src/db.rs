use anyhow::Result;
use sqlx::{Pool, Postgres, types::Uuid};

/// Updates the Noir 1024-bit verifier contract address in the database
pub async fn update_noir_verifier_contract_address_1024(
    pool: &Pool<Postgres>,
    id: &Uuid,
    address: &str,
) -> Result<()> {
    let query = r#"
        UPDATE blueprints
        SET noir_verifier_contract_address_1024 = $1
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(address)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Updates the Noir 2048-bit verifier contract address in the database
pub async fn update_noir_verifier_contract_address_2048(
    pool: &Pool<Postgres>,
    id: &Uuid,
    address: &str,
) -> Result<()> {
    let query = r#"
        UPDATE blueprints
        SET noir_verifier_contract_address_2048 = $1
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(address)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
