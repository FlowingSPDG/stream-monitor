/// SqlTemplateRepository - sql_templates テーブル専用レポジトリ
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlTemplate {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub query: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct SqlTemplateRepository;

const SELECT_COLUMNS: &str = "SELECT id, name, description, query, 
     CAST(created_at AS VARCHAR) as created_at, 
     CAST(updated_at AS VARCHAR) as updated_at";

impl SqlTemplateRepository {
    /// 全SQLテンプレートを取得（updated_at 降順）
    pub fn list_all(conn: &Connection) -> Result<Vec<SqlTemplate>, duckdb::Error> {
        let mut stmt = conn.prepare(&format!(
            "{} FROM sql_templates ORDER BY updated_at DESC",
            SELECT_COLUMNS
        ))?;
        let rows = stmt.query_map([], |row| {
            Ok(SqlTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).unwrap_or_else(|_| String::new()),
                query: row.get(3)?,
                created_at: row.get(4).unwrap_or_else(|_| String::new()),
                updated_at: row.get(5).unwrap_or_else(|_| String::new()),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    /// IDでテンプレートを取得
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<SqlTemplate>, duckdb::Error> {
        let mut stmt = conn.prepare(&format!(
            "{} FROM sql_templates WHERE id = ?",
            SELECT_COLUMNS
        ))?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(SqlTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).unwrap_or_else(|_| String::new()),
                query: row.get(3)?,
                created_at: row.get(4).unwrap_or_else(|_| String::new()),
                updated_at: row.get(5).unwrap_or_else(|_| String::new()),
            })
        })?;
        rows.next().transpose()
    }

    /// テンプレートを保存（id > 0 の場合は更新、0 の場合は新規作成）
    pub fn save(
        conn: &Connection,
        id: i64,
        name: &str,
        description: &str,
        query: &str,
    ) -> Result<SqlTemplate, duckdb::Error> {
        let id = if id > 0 {
            conn.execute(
                "UPDATE sql_templates 
                 SET name = ?, description = ?, query = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ?",
                params![name, description, query, id],
            )?;
            id
        } else {
            conn.execute(
                "INSERT INTO sql_templates (name, description, query) VALUES (?, ?, ?)",
                params![name, description, query],
            )?;
            let mut stmt = conn.prepare("SELECT currval('sql_templates_id_seq')")?;
            stmt.query_row([], |row| row.get(0))?
        };

        Self::get_by_id(conn, id)?.ok_or(duckdb::Error::QueryReturnedNoRows)
    }

    /// テンプレートを削除。削除した行数を返す（0の場合は未存在）
    pub fn delete(conn: &Connection, id: i64) -> Result<u64, duckdb::Error> {
        let affected = conn.execute("DELETE FROM sql_templates WHERE id = ?", params![id])?;
        Ok(affected as u64)
    }
}
