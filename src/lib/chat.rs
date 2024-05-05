#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Message {
    pub from: User,
    pub id: u32,
    pub content: String,
}
