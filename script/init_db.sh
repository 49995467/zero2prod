#!/usr/bin/env bash
set -x
set -eo pipefail

# 检查是否已安装psql和sqlx, -x 参数用于检查命令是否存在,command -v psql 用于检查是否安装了psql
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  echo
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "Run cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

# 检查是否已设置自定义的数据库用户名。如果没有，则使用默认的用户名postgres，:= 操作符表示如果变量未设置，则将其设置为指定的值
DB_USER=${POSTGRES_USER:=postgres}
# 检查是否已设置自定义的数据库密码。如果没有，则使用默认的密码password
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
# 检查是否已设置自定义的数据库名称。如果没有，则使用默认的名称newsletter
DB_NAME=${POSTGRES_DB:=newsletter}
# 检查是否已设置自定义的数据库端口。如果没有，则使用默认的端口5432
DB_PORT=${POSTGRES_PORT:=5432}

# 检查是否已设置SKIP_DOCKER环境变量。如果已设置，则不使用Docker启动Postgres数据库.-z 参数用于检查变量是否为空
if [[ -z "$SKIP_DOCKER" ]]
then
# 使用Docker启动Postgres数据库
docker run \
  -e POSTGRES_USER="${DB_USER}" \
  -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
  -e POSTGRES_DB="${DB_NAME}" \
  -p "${DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000
fi

  #保持对Postgres数据库的轮询，直到它准备好接受命令
  export PGPASSWORD="${DB_PASSWORD}"
  until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
  done

  >&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

  export DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
  sqlx database create
  sqlx migrate run

  >&2 echo "Postgres has been migrated, ready to go!"