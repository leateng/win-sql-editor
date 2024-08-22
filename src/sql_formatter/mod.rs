use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

pub fn format_sql(sql: &str) -> String {
    let dialect = PostgreSqlDialect {};
    let asts = Parser::parse_sql(&dialect, sql).unwrap();
    let mut sql_vec: Vec<String> = Vec::new();
    for ast in asts.iter() {
        sql_vec.push(format!("{}", ast));
    }

    sql_vec.join(";\n")
}
