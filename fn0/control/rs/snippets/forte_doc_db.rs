#[forte_doc]
pub struct User {
    #[pk]
    pub id: String,
    pub name: String,
}

// write
UserPut(user).send_with(&db).await?;

// read
let user = UserGet { id }.send_with(&db).await?;

// transaction with conflict retry
db.trx(|trx| async move {
    let mut user = trx
        .get(UserGet { id })
        .await?
        .ok_or_else(|| anyhow!("not found"))?;
    user.name = "New Name".into();
    trx.commit(())
}).await;
