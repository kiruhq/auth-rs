use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.as_slice() {
        [command] if command == "generate" => {
            print!("{}", auth_rs::schema::postgres_base_schema_sql());
            ExitCode::SUCCESS
        }
        [command, database] if command == "generate" && database == "postgres" => {
            print!("{}", auth_rs::schema::postgres_base_schema_sql());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: auth-rs generate [postgres]");
            ExitCode::from(2)
        }
    }
}
