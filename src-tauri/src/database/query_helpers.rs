/// DuckDB特殊型を扱うためのSQLフラグメント生成
/// 
/// DuckDBのLIST型やTIMESTAMP型を直接SELECTすると、Rustのduckdbクレートで
/// 型変換エラーが発生します。このモジュールは、それらの型を安全に扱うための
/// SQLフラグメントを生成します。
pub mod chat_query {
    /// badgesカラムのSELECT句（常にVARCHARにキャスト）
    /// 
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}", chat_query::badges_select("cm"));
    /// // 生成されるSQL: "SELECT CAST(cm.badges AS VARCHAR) as badges"
    /// ```
    pub fn badges_select(table_alias: &str) -> String {
        format!("CAST({}.badges AS VARCHAR) as badges", table_alias)
    }
    
    /// timestampカラムのSELECT句（常にVARCHARにキャスト）
    /// 
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}", chat_query::timestamp_select("cm"));
    /// // 生成されるSQL: "SELECT CAST(cm.timestamp AS VARCHAR) as timestamp"
    /// ```
    pub fn timestamp_select(table_alias: &str) -> String {
        format!("CAST({}.timestamp AS VARCHAR) as timestamp", table_alias)
    }
    
    /// chat_messagesの基本SELECT句（よく使うカラムセット）
    /// 
    /// DuckDB特殊型（badges, timestamp）を含む標準的なカラムセットを生成します。
    /// 
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {} FROM chat_messages cm", 
    ///                   chat_query::standard_columns("cm"));
    /// ```
    pub fn standard_columns(table_alias: &str) -> String {
        format!(
            "{}.id, {}.channel_id, {}.stream_id, {}, {}.platform, \
             {}.user_id, {}.user_name, {}.message, {}.message_type, {}, {}.badge_info",
            table_alias,
            table_alias,
            table_alias,
            timestamp_select(table_alias),
            table_alias,
            table_alias,
            table_alias,
            table_alias,
            table_alias,
            badges_select(table_alias),
            table_alias
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badges_select() {
        let sql = chat_query::badges_select("cm");
        assert_eq!(sql, "CAST(cm.badges AS VARCHAR) as badges");
    }

    #[test]
    fn test_timestamp_select() {
        let sql = chat_query::timestamp_select("cm");
        assert_eq!(sql, "CAST(cm.timestamp AS VARCHAR) as timestamp");
    }

    #[test]
    fn test_standard_columns() {
        let sql = chat_query::standard_columns("cm");
        assert!(sql.contains("cm.id"));
        assert!(sql.contains("CAST(cm.badges AS VARCHAR) as badges"));
        assert!(sql.contains("CAST(cm.timestamp AS VARCHAR) as timestamp"));
    }
}
