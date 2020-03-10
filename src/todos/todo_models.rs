use chrono::prelude::*;
use postgres::types::ToSql;
use postgres::{self, NoTls, Row};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Todo {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
    creation_date: DateTime<Utc>,
    last_edit_date: DateTime<Utc>,
    done: bool,
}

impl Todo {
    pub fn new(
        id: i32,
        user_id: i32,
        title: String,
        body: Option<String>,
        creation_date: DateTime<Utc>,
        last_edit_date: DateTime<Utc>,
        done: bool,
    ) -> Self {
        Todo {
            id,
            user_id,
            title,
            body,
            creation_date,
            last_edit_date,
            done,
        }
    }
}

pub struct TodoDbExecutor {
    connection: PooledConnection<PostgresConnectionManager<NoTls>>,
}

impl TodoDbExecutor {
    pub fn new(connection: PooledConnection<PostgresConnectionManager<NoTls>>) -> Self {
        TodoDbExecutor { connection }
    }

    pub fn create(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "INSERT INTO todo(user_id, title, body) VALUES($1, $2, $3) RETURNING id, creation_date",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }

    pub fn get(&mut self, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.query(
            "SELECT id, user_id, title, body, creation_date, last_edit_date, done FROM todo WHERE user_id = $1 ORDER BY last_edit_date DESC OFFSET $2 LIMIT $3",
            params,
        )?;
        transaction.commit()?;

        Ok(rows)
    }

    pub fn delete_all(&mut self) -> Result<u64, postgres::Error> {
        let mut transaction = self.connection.transaction()?;
        let rows = transaction.execute("DELETE FROM todo", &[])?;
        transaction.commit()?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::Pool;

    fn create_connection() -> Pool<PostgresConnectionManager<NoTls>> {
        let manager = PostgresConnectionManager::new(
            "host=localhost user=dshur dbname=productivity password=1234"
                .parse()
                .unwrap(),
            postgres::NoTls,
        );

        Pool::new(manager).expect("Couldn't create a new connection pool in create_connection")
    }

    #[test]
    fn test_create_todo() {
        let pool = create_connection();

        let connection = pool
            .get()
            .expect("Could not get a connection from the pool in test_create_todo");

        let mut executor = TodoDbExecutor::new(connection);

        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");
        executor
            .create(&[&1, &"title1".to_owned(), &"body1".to_owned()])
            .expect("Couldn't create todo");

        let rows = executor
            .get(&[&0, &None::<i64>, &None::<i64>])
            .expect("Couldn't get todos");
        assert_eq!(rows.len(), 0);

        let rows = executor
            .get(&[&1, &None::<i64>, &None::<i64>])
            .expect("Couldn't get todos");
        assert_eq!(rows.len(), 10);

        let rows = executor
            .get(&[&1, &None::<i64>, &(5 as i64)])
            .expect("Couldn't get todos");
        assert_eq!(rows.len(), 5);

        let count = executor.delete_all().expect("Couldn't delete the created todos");
        assert_eq!(count, 10);
    }
}
