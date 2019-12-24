pub enum GroupOp {
    AND,
    OR,
}

pub enum Condition {
    Group(GroupOp, Vec<Condition>),
    Term(String, String),
}

pub struct Query {
    head: Condition,
}

impl From<Condition> for Query {
    fn from(c: Condition) -> Query {
        Query { head: c }
    }
}

impl Query {
    pub fn root(&self) -> &Condition {
        &self.head
    }
}
