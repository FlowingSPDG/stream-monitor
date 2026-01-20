use duckdb::{Connection, Result as DuckResult};

/// DuckDBの動的パラメータを処理するヘルパー関数
/// パラメータが0-15個の場合にのみサポート
pub fn execute_with_params(conn: &Connection, sql: &str, params: &[String]) -> DuckResult<usize> {
    match params.len() {
        0 => conn.execute(sql, []),
        1 => conn.execute(sql, [params[0].as_str()]),
        2 => conn.execute(sql, [params[0].as_str(), params[1].as_str()]),
        3 => conn.execute(
            sql,
            [params[0].as_str(), params[1].as_str(), params[2].as_str()],
        ),
        4 => conn.execute(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
            ],
        ),
        5 => conn.execute(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
                params[4].as_str(),
            ],
        ),
        _ => Err(duckdb::Error::InvalidParameterName(
            "Too many parameters (max 5 supported)".to_string(),
        )),
    }
}

pub fn query_map_with_params<'stmt, T, F>(
    stmt: &'stmt mut duckdb::Statement,
    params: &[String],
    f: F,
) -> DuckResult<duckdb::MappedRows<'stmt, F>>
where
    F: FnMut(&duckdb::Row) -> DuckResult<T>,
{
    match params.len() {
        0 => stmt.query_map([], f),
        1 => stmt.query_map([params[0].as_str()], f),
        2 => stmt.query_map([params[0].as_str(), params[1].as_str()], f),
        3 => stmt.query_map(
            [params[0].as_str(), params[1].as_str(), params[2].as_str()],
            f,
        ),
        4 => stmt.query_map(
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
            ],
            f,
        ),
        5 => stmt.query_map(
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
                params[4].as_str(),
            ],
            f,
        ),
        _ => Err(duckdb::Error::InvalidParameterName(
            "Too many parameters (max 5 supported)".to_string(),
        )),
    }
}
