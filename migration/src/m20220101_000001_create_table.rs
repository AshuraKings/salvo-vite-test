use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use sea_orm_migration::{prelude::*, schema::*};

use crate::models::users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("users")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string("name").not_null())
                    .col(string("username").not_null())
                    .col(string("passwd").text().not_null())
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        let roles = ["super_admin", "admin", "custom"];
        for role in roles.map(|r| r.to_string()) {
            let name = role.clone();
            let username = role.clone();
            let passwd = format!("{role}@123");
            users::ActiveModel {
                name: Set(name),
                username: Set(username),
                passwd: Set(passwd),
                id: Set(uuid::Uuid::now_v7()),
            }
            .insert(db)
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("users").if_exists().to_owned())
            .await
    }
}
