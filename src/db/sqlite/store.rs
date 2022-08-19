use sqlite::Connection;

use crate::store::{Read, Store, Transaction as StoreTransaction, World, Write};
const LIBNAME: &str = "library";
const CREATE_TABLE: &str = r#"CREATE TABLE  IF NOT EXISTS library (
      word TEXT PRIMARY KEY,
      message TEXT NOT NULL
    )"#;

use super::*;

extern crate sqlite;

pub struct Sqlite {
    _path: String,
    connect: sqlite::Connection,
    _insert_stmt: String,
    _delete_stmt: String,
    _update_stmt: String,
    _get_stmt: String,
    _dump_stmt: String,
    _dump_size_stmt: String,
}

impl Sqlite {
    fn create_table(&self) -> Result<()> {
        let con = &self.connect;
        con.execute(CREATE_TABLE)?;
        Ok(())
    }

    pub fn new(path: String) -> Result<Sqlite> {
        let s = Sqlite::new_conn(path)?;
        println!("create table");
        s.create_table()?;
        Ok(s)
    }

    fn new_conn(path: String) -> Result<Sqlite> {
        let con = Connection::open_with_flags(
            &path,
            sqlite::OpenFlags::new()
                // .set_full_mutex()
                // .set_no_mutex()
                .set_read_write()
                .set_create(),
        )?;

        let s = Sqlite {
            connect: con,
            _path: path,
            _insert_stmt: format!("INSERT INTO {} (word, message) VALUES (?, ?);", LIBNAME),
            _delete_stmt: format!("DELETE FROM {} WHERE word=?", LIBNAME),
            _update_stmt: format!("UPDATE {} SET message=? WHERE word=?;", LIBNAME),
            _get_stmt: format!("SELECT message FROM {} WHERE word=?", LIBNAME),
            _dump_stmt: format!("SELECT word, message from {}", LIBNAME),
            _dump_size_stmt: format!("SELECT word, message from {} LIMIT ?", LIBNAME),
        };
        Ok(s)
    }
}

impl Write<World, InternalError<Error>> for Sqlite {
    fn insert(&self, word: &World) -> Result<Option<World>> {
        let con = &self.connect;
        let mut stmt = con
            .prepare(self._insert_stmt.as_str())?
            // .bind(0, LIBNAME)?
            .bind(1, word.name.as_str())?
            .bind(2, word.message.as_str())?;
        stmt.next()?;

        Ok(None)
    }
    fn update(&self, word: &World) -> Result<Option<World>> {
        let mut stmt = self
            .connect
            .prepare(self._update_stmt.as_str())?
            .bind(1, word.message.as_str())?
            .bind(2, word.name.as_str())?;
        stmt.next()?;

        Ok(None)
    }
    fn delete(&self, word: &str) -> Result<Option<World>> {
        let con = &self.connect;
        let mut stmt = con
            .prepare(self._delete_stmt.as_str())?
            // .bind(1, LIBNAME)?
            .bind(1, word)?;
        stmt.next()?;
        Ok(None)
    }
}

impl Read<World, InternalError<Error>> for Sqlite {
    fn get(&self, id: &str) -> Result<World> {
        let mut stmt = self.connect.prepare(self._get_stmt.as_str())?.bind(1, id)?;

        let state = stmt.next()?;
        match state {
            sqlite::State::Done => {
                // Err(RCStoreError::new(1, String::from("qwe")))
                Err(InternalError::NotFound)
            }
            sqlite::State::Row => Ok(World {
                name: id.to_string(),
                message: stmt.read(0)?,
            }),
        }
    }

    // WARN memory head may be very big
    // TODO add a iterator for batch select sql
    fn list(&self, size: i64) -> Result<Vec<World>> {
        let mut stmt: sqlite::Statement;
        if size <= 0 {
            stmt = self.connect.prepare(self._dump_stmt.as_str())?
        } else {
            stmt = self
                .connect
                .prepare(self._dump_size_stmt.as_str())?
                .bind(1, size)?;
        }

        let mut res: Vec<World> = Vec::new();
        loop {
            match stmt.next()? {
                sqlite::State::Done => {
                    break;
                }
                sqlite::State::Row => res.push(World {
                    name: stmt.read(0)?,
                    message: stmt.read(1)?,
                }),
            }
        }

        Ok(res)
    }
}

impl Store<World, InternalError<Error>, Tansaction> for Sqlite {
    fn begin(&self) -> Result<Tansaction> {
        let s = Sqlite::new_conn(String::from(self._path.as_str()))?;
        {
            let con = &s.connect;
            let mut state = con.prepare("BEGIN")?;
            state.next()?;
        }

        Ok(Tansaction { stmt: s })
    }
}

pub struct Tansaction {
    // pub stmt: &'a super::store::Sqlite,
    pub stmt: Sqlite,
}

impl Read<World, InternalError<Error>> for Tansaction {
    fn get(&self, id: &str) -> Result<World> {
        self.stmt.get(id)
    }

    // WARN memory head may be very big
    // TODO add a iterator for batch select sql
    fn list(&self, size: i64) -> Result<Vec<World>> {
        self.stmt.list(size)
    }
}

impl<'a> Write<World, InternalError<Error>> for Tansaction {
    fn insert(&self, word: &World) -> Result<Option<World>> {
        self.stmt.insert(word)
    }

    fn update(&self, word: &World) -> Result<Option<World>> {
        self.stmt.update(word)
    }

    fn delete(&self, word: &str) -> Result<Option<World>> {
        self.stmt.delete(word)
    }
}

impl<'a> StoreTransaction<World, InternalError<Error>> for Tansaction {
    fn rollback(self) -> std::result::Result<(), InternalError<Error>> {
        self.stmt.connect.execute("ROLLBACK")?;
        Ok(())
    }

    fn commit(self) -> std::result::Result<(), InternalError<Error>> {
        self.stmt.connect.execute("COMMIT")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    fn new_sqlite() -> Sqlite {
        Sqlite::new(String::from("target/test.sql")).unwrap()
    }

    fn clean_sqlite() {
        std::fs::remove_file("target/test.sql").unwrap();
    }

    fn expect_notfound<T: Debug>(err: Result<T>) {
        match err.expect_err("no error happen") {
            InternalError::NotFound => {}
            err => {
                panic!("{}", err)
            }
        }
    }

    fn expect_unknow<T: Debug>(err: Result<T>) -> Error {
        match err.expect_err("no error happen") {
            InternalError::Unknow(err) => err.err.unwrap(),
            err => {
                panic!("shuold input a unkonw error, {}", err)
            }
        }
    }

    #[test]
    fn transaction() {
        let sql = new_sqlite();
        let word = World {
            name: String::from("test"),
            message: String::from("test"),
        };
        sql.insert(&word).unwrap();

        // rollback
        let tran = sql.begin().unwrap();
        tran.delete(&word.message).unwrap();
        expect_notfound(tran.stmt.get(&word.name));
        tran.rollback().unwrap();
        sql.get(&word.name).unwrap();

        //commit
        let tran = sql.begin().unwrap();
        tran.delete(&word.name).unwrap();
        expect_notfound(tran.stmt.get(&word.name));
        tran.commit().unwrap();
        expect_notfound(sql.get(&word.name));

        clean_sqlite();
    }

    #[test]
    fn gen_data() {
        let sql = Sqlite::new(String::from("/mnt/d/big_test.sql")).unwrap();
        let mut word = World {
            name: String::from("test"),
            message: String::from("test"),
        };

        let size = 1000;
        let mut cost = std::time::Duration::new(0, 0);
        for i in 0..size {
            word.name = format!("test-{}", i);
            let now = std::time::Instant::now();
            sql.insert(&word).unwrap();
            cost += now.elapsed();
        }

        println!("cost {}", cost.as_secs());

        word.name = format!("test-big-message");
        let content = std::fs::read("src/db/sqlite/store.rs").unwrap();
        word.message = String::from_utf8(content).unwrap();
        sql.insert(&word).unwrap();
    }

    #[test]
    fn store() {
        let sql = new_sqlite();
        let mut word = World {
            name: String::from("test"),
            message: String::from("test"),
        };

        sql.insert(&word).unwrap();

        // primary key conflict error
        let err = expect_unknow(sql.insert(&word));
        assert_eq!(err.code.unwrap() as i32, sqlite3_sys::SQLITE_CONSTRAINT);

        word.message = String::from("test update");
        sql.update(&word).unwrap();

        // test get
        let mut word1 = sql.get(&word.name).unwrap();
        assert_eq!(word.message, word1.message);

        // test list
        word1.name = String::from("test1");
        sql.insert(&word1).unwrap();

        let ws = sql.list(0).unwrap();
        assert_eq!(ws.len(), 2);

        let ws = sql.list(1).unwrap();
        assert_eq!(ws.len(), 1);

        // test delete
        sql.delete(&word.name).unwrap();

        // test get not found
        let err = sql.get("nofound").expect_err("notfound errir");
        assert!(matches!(err, InternalError::NotFound));

        // clean
        clean_sqlite();
    }
}
