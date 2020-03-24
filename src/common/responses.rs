use serde::Serialize;

#[derive(Serialize)]
pub struct ServerResponse<T: Serialize, U> {
    data: T,
    meta: U,
}

impl<T, U> ServerResponse<T, U>
where
    T: Serialize,
    U: Serialize,
{
    pub fn new(data: T, meta: U) -> Self {
        ServerResponse { data, meta }
    }
}
