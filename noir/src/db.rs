use anyhow::Result;
use sqlx::{Pool, Postgres, types::Uuid};

/// Updates the verifier contract address in the database
pub async fn update_verifier_contract_address(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: &str,
) -> Result<()> {
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

/// Updates the contract address in the database for integer IDs
pub async fn update_contract_address(
    pool: &Pool<Postgres>,
    blueprint_id: i32,
    contract_address: &str,
) -> Result<()> {
    let query = r#"
        UPDATE blueprints 
        SET noir_contract_address = $1 
        WHERE id = $2
    "#;

    sqlx::query(query)
        .bind(contract_address)
        .bind(blueprint_id)
        .execute(pool)
        .await?;

    Ok(())
}
