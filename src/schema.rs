use sea_query::{
    ColumnDef, Expr, ForeignKey, ForeignKeyAction, Index, PostgresQueryBuilder, Table,
};

pub fn postgres_base_schema_sql() -> String {
    let statements = base_postgres_schema_statements();

    format!("{};\n", statements.join(";\n\n"))
}

pub fn base_postgres_schema_statements() -> Vec<String> {
    vec![
        user_table().to_string(PostgresQueryBuilder),
        account_table().to_string(PostgresQueryBuilder),
        session_table().to_string(PostgresQueryBuilder),
        verification_table().to_string(PostgresQueryBuilder),
        pending_signup_table().to_string(PostgresQueryBuilder),
        account_provider_account_id_index().to_string(PostgresQueryBuilder),
        account_user_id_index().to_string(PostgresQueryBuilder),
        session_user_id_index().to_string(PostgresQueryBuilder),
        verification_kind_identifier_index().to_string(PostgresQueryBuilder),
    ]
}

fn user_table() -> sea_query::TableCreateStatement {
    Table::create()
        .table("user")
        .if_not_exists()
        .col(text_primary_key("id"))
        .col(text_not_null("name"))
        .col(text_not_null("email").unique_key())
        .col(
            ColumnDef::new("email_verified")
                .boolean()
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new("image").text())
        .col(timestamp_column("created_at"))
        .col(timestamp_column("updated_at"))
        .to_owned()
}

fn account_table() -> sea_query::TableCreateStatement {
    Table::create()
        .table("account")
        .if_not_exists()
        .col(text_primary_key("id"))
        .col(text_not_null("account_id"))
        .col(text_not_null("user_id"))
        .col(text_not_null("provider_id"))
        .col(ColumnDef::new("password").text())
        .col(timestamp_column("created_at"))
        .col(timestamp_column("updated_at"))
        .foreign_key(
            ForeignKey::create()
                .name("account_user_id_fkey")
                .from("account", "user_id")
                .to("user", "id")
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

fn session_table() -> sea_query::TableCreateStatement {
    Table::create()
        .table("session")
        .if_not_exists()
        .col(text_primary_key("id"))
        .col(text_not_null("user_id"))
        .col(text_not_null("token").unique_key())
        .col(
            ColumnDef::new("expires_at")
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(ColumnDef::new("ip_address").text())
        .col(ColumnDef::new("user_agent").text())
        .col(timestamp_column("created_at"))
        .col(timestamp_column("updated_at"))
        .foreign_key(
            ForeignKey::create()
                .name("session_user_id_fkey")
                .from("session", "user_id")
                .to("user", "id")
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

fn verification_table() -> sea_query::TableCreateStatement {
    Table::create()
        .table("verification")
        .if_not_exists()
        .col(text_primary_key("id"))
        .col(text_not_null("kind"))
        .col(text_not_null("identifier"))
        .col(text_not_null("token_hash").unique_key())
        .col(
            ColumnDef::new("expires_at")
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(timestamp_column("created_at"))
        .col(timestamp_column("updated_at"))
        .to_owned()
}

fn pending_signup_table() -> sea_query::TableCreateStatement {
    Table::create()
        .table("pending_signup")
        .if_not_exists()
        .col(text_primary_key("id"))
        .col(text_not_null("user_id"))
        .col(text_not_null("account_id"))
        .col(text_not_null("email").unique_key())
        .col(text_not_null("name"))
        .col(text_not_null("password_hash"))
        .col(ColumnDef::new("image").text())
        .col(timestamp_column("created_at"))
        .col(timestamp_column("updated_at"))
        .to_owned()
}

fn account_provider_account_id_index() -> sea_query::IndexCreateStatement {
    Index::create()
        .name("account_provider_account_id_key")
        .table("account")
        .if_not_exists()
        .col("provider_id")
        .col("account_id")
        .unique()
        .to_owned()
}

fn account_user_id_index() -> sea_query::IndexCreateStatement {
    Index::create()
        .name("account_user_id_idx")
        .table("account")
        .if_not_exists()
        .col("user_id")
        .to_owned()
}

fn session_user_id_index() -> sea_query::IndexCreateStatement {
    Index::create()
        .name("session_user_id_idx")
        .table("session")
        .if_not_exists()
        .col("user_id")
        .to_owned()
}

fn verification_kind_identifier_index() -> sea_query::IndexCreateStatement {
    Index::create()
        .name("verification_kind_identifier_idx")
        .table("verification")
        .if_not_exists()
        .col("kind")
        .col("identifier")
        .to_owned()
}

fn text_primary_key(name: &'static str) -> ColumnDef {
    let mut column = ColumnDef::new(name);
    column.text().not_null().primary_key();
    column
}

fn text_not_null(name: &'static str) -> ColumnDef {
    let mut column = ColumnDef::new(name);
    column.text().not_null();
    column
}

fn timestamp_column(name: &'static str) -> ColumnDef {
    let mut column = ColumnDef::new(name);
    column
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    column
}
