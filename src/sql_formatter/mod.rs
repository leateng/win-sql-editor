use sqlformat::{format, FormatOptions, Indent, QueryParams};
// use sqlparser::ast::Statement;
// use sqlparser::dialect::PostgreSqlDialect;
// use sqlparser::parser::Parser;
// use sqlparser::parser::ParserError;

// pub fn format_sql(sql: &str) -> Result<Vec<Statement>, ParserError> {
//     let dialect = PostgreSqlDialect {};
//     let asts = Parser::parse_sql(&dialect, sql);
//     return asts;
//     // let mut sql_vec: Vec<String> = Vec::new();
//     // for ast in asts.iter() {
//     //     sql_vec.push(format!("{}", ast));
//     // }
//     //
//     // sql_vec.join(";\n")
// }

pub fn format_sql(sql: &str) -> String {
    let format_options = FormatOptions {
        indent: Indent::Spaces(2),
        uppercase: true,
        lines_between_queries: 2,
    };
    let formatted_sql = format(sql, &QueryParams::None, format_options);
    return formatted_sql;
}
