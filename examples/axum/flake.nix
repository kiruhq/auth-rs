{
  description = "auth-rs Axum example development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

      postgresPort = "55432";
      databaseName = "auth_rs_axum";
      databaseUser = "postgres";
      databasePassword = "postgres";
      databaseUrl = "postgres://${databaseUser}:${databasePassword}@127.0.0.1:${postgresPort}/${databaseName}";
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          db = pkgs.writeShellApplication {
            name = "axum-auth-db";
            runtimeInputs = [
              pkgs.coreutils
              pkgs.gnugrep
              pkgs.postgresql_16
            ];
            text = ''
              set -euo pipefail

              data_dir="''${PGDATA:-.postgres}"
              host="''${PGHOST:-127.0.0.1}"
              port="''${PGPORT:-${postgresPort}}"
              db_name="''${PGDATABASE:-${databaseName}}"
              db_user="''${PGUSER:-${databaseUser}}"
              db_password="''${PGPASSWORD:-${databasePassword}}"
              database_url="postgres://$db_user:$db_password@$host:$port/$db_name"

              if [ ! -s "$data_dir/PG_VERSION" ]; then
                initdb --username="$db_user" --auth=trust --encoding=UTF8 --locale=C "$data_dir"
                {
                  echo "listen_addresses = '$host'"
                  echo "port = $port"
                } >> "$data_dir/postgresql.conf"
                {
                  echo "host all all 127.0.0.1/32 trust"
                  echo "host all all ::1/128 trust"
                } >> "$data_dir/pg_hba.conf"
              fi

              if pg_ctl -D "$data_dir" status >/dev/null 2>&1; then
                echo "Postgres is already running for $data_dir"
                echo "DATABASE_URL=$database_url"
                exit 0
              fi

              if pg_isready -h "$host" -p "$port" >/dev/null 2>&1; then
                echo "A Postgres server is already accepting connections at $host:$port." >&2
                echo "Stop that server or choose a different port with PGPORT=..." >&2
                exit 1
              fi

              pg_ctl -D "$data_dir" -o "-c listen_addresses=$host -c port=$port" -w start
              started_for_setup=1

              cleanup_setup() {
                if [ "$started_for_setup" = "1" ]; then
                  pg_ctl -D "$data_dir" -m fast -w stop >/dev/null
                fi
              }

              trap cleanup_setup EXIT

              psql -h "$host" -p "$port" -U "$db_user" -d postgres -v ON_ERROR_STOP=1 \
                -c "ALTER USER \"$db_user\" WITH PASSWORD '$db_password';" >/dev/null

              if ! psql -h "$host" -p "$port" -U "$db_user" -d postgres -tAc \
                "SELECT 1 FROM pg_database WHERE datname = '$db_name'" | grep -q 1; then
                createdb -h "$host" -p "$port" -U "$db_user" "$db_name"
              fi

              pg_ctl -D "$data_dir" -m fast -w stop >/dev/null
              started_for_setup=0
              trap - EXIT

              echo "Postgres is running on $host:$port"
              echo "DATABASE_URL=$database_url"
              echo "Press Ctrl-C to stop."

              exec postgres -D "$data_dir" -c "listen_addresses=$host" -c "port=$port"
            '';
          };
        in
        {
          default = db;
          db = db;
        }
      );

      apps = forAllSystems (
        system:
        let
          dbProgram = "${self.packages.${system}.db}/bin/axum-auth-db";
        in
        {
          default = {
            type = "app";
            program = dbProgram;
            meta.description = "Start the local Postgres database for the Axum auth-rs example";
          };
          db = {
            type = "app";
            program = dbProgram;
            meta.description = "Start the local Postgres database for the Axum auth-rs example";
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.openssl
              pkgs.pkg-config
              pkgs.postgresql_16
              pkgs.rustc
              pkgs.sqlx-cli
            ];

            DATABASE_URL = databaseUrl;

            shellHook = ''
              export DATABASE_URL="${databaseUrl}"
              echo "DATABASE_URL=$DATABASE_URL"
              echo "Start database: nix run .#db"
              echo 'Apply schema: cargo run --manifest-path ../../Cargo.toml --bin auth-rs -- generate postgres | psql "$DATABASE_URL"'
              echo "Run example: cargo run"
            '';
          };
        }
      );
    };
}
