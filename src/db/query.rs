//! Query Builder Module
#![allow(dead_code)]

/// Query builder for constructing database queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_clauses: Vec<WhereClause>,
    order_by: Vec<(String, Order)>,
    limit: Option<u32>,
    offset: Option<u32>,
    joins: Vec<Join>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub field: String,
    pub operator: Operator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct Join {
    pub table: String,
    pub on: String,
    pub join_type: JoinType,
}

#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn table(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select_fields: vec!["*".to_string()],
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    /// Select specific fields
    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a where clause
    pub fn where_eq(mut self, field: &str, value: impl ToString) -> Self {
        self.where_clauses.push(WhereClause {
            field: field.to_string(),
            operator: Operator::Eq,
            value: value.to_string(),
        });
        self
    }

    /// Add order by clause
    pub fn order_by(mut self, field: &str, order: Order) -> Self {
        self.order_by.push((field.to_string(), order));
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build the SQL query
    pub fn build(&self) -> String {
        let mut sql = format!(
            "SELECT {} FROM {}",
            self.select_fields.join(", "),
            self.table
        );

        if !self.where_clauses.is_empty() {
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|w| format!("{} = '{}'", w.field, w.value))
                .collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        if !self.order_by.is_empty() {
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|(f, o)| {
                    format!(
                        "{} {}",
                        f,
                        match o {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", orders.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}
