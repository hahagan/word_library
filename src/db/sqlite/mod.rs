
use sqlite::{Connection, Error};
use std::fmt;
// use crate::store::RCError as RCStoreError;
use crate::store::{Error as StoreError};



use crate::store::{Store, World};
const LIBNAME:&str = "library";
const CREATE_TABLE:&str = r#"CREATE TABLE  IF NOT EXISTS library (
      word TEXT PRIMARY KEY,
      message TEXT NOT NULL
    )"#;

extern crate sqlite;

type _Error = InternalError<StoreError<Error>>;
pub type Result<T> = std::result::Result<T, _Error>;

#[derive(Debug)]
pub enum InternalError<T>{
    NotFound,
    Unknow(T),
}

impl<T:std::fmt::Debug> std::error::Error for InternalError<T> {}
impl<T> fmt::Display for InternalError<T> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "internal error")
    }
}


impl From<Error> for _Error{
  fn from(err: Error) -> Self {
    InternalError::Unknow(StoreError { message: String::from("unKnow"), code: 0, err: Some(err) })
}
}

struct Sqlite {
  connect: sqlite::Connection,
  _insert_stmt: String,
  _delete_stmt: String,
  _update_stmt: String,
  _get_stmt: String,
}


impl Sqlite {
  fn create_table(&self)  -> Result<()> {
    let con = &self.connect;
    con.execute(CREATE_TABLE)?;
    Ok(())
  }
}

impl Sqlite {

  pub fn new(path:String) -> Result<Sqlite> {
    
    let con = Connection::open_with_flags(path, 
      sqlite::OpenFlags::new().set_no_mutex().set_read_write().set_create())?;

    let s = Sqlite{
      connect: con,
      _insert_stmt: format!("INSERT INTO {} (word, message) VALUES (?, ?);", LIBNAME),
      _delete_stmt: format!("DELETE FROM {} WHERE word=?", LIBNAME),
      _update_stmt: format!("UPDATE {} SET message=? WHERE word=?;", LIBNAME),
      _get_stmt: format!("SELECT message FROM {} WHERE word=?", LIBNAME)
    };
    s.create_table()?;

    Ok(s)
  }
}


// lazy_static!{
//   static ref init:i32 = {
//     unsafe {
//       NOT_FOUND_ERROR = Some(StoreError::new(0, String::from("notfound"))); 
//     }
//     1
//   };
// }

// static NOT_FOUND_ERROR:_Error = StoreError::newInternal(0);

impl Store<(), _Error> for Sqlite {
  fn insert(&self, word: &World) -> Result<()> {
    let con = &self.connect;
    let mut stmt = con.prepare(self._insert_stmt.as_str())?
    // .bind(0, LIBNAME)?
    .bind(1, word.name.as_str())?
    .bind(2, word.message.as_str())?;
    stmt.next()?;

    Ok(())
  }
  fn update(&self, word: &World) ->  Result<()> {
    let mut stmt = self.connect.prepare(self._update_stmt.as_str())?
    .bind(1, word.message.as_str())?
    .bind(2, word.name.as_str())?;
    stmt.next()?;
    
    Ok(())

  }
  fn delete(&self, word: &str) ->  Result<()>{
    let con = &self.connect;
    let mut stmt = con.prepare(self._delete_stmt.as_str())?
    // .bind(1, LIBNAME)?
    .bind(1, word)?;
    stmt.next()?;
    Ok(())
  }
  fn get(&self, id: &str) -> Result<World> {
    let mut stmt = self.connect.prepare(self._get_stmt.as_str())?
    .bind(1, id)?;

    let state = stmt.next()?;
    match state {
        sqlite::State::Done => {
          // Err(RCStoreError::new(1, String::from("qwe")))
          Err(InternalError::NotFound)
        }
        sqlite::State::Row => {
          Ok(World{
            name: id.to_string(),
            message: stmt.read(0)?,
          })
        }
    }

  }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sql = Sqlite::new(String::from("target/test.sql")).unwrap();
        let mut word = World { name: String::from("test"), message: String::from("test") };

        sql.insert(&word).unwrap();
        word.message = String::from("test update");
        sql.update(&word).unwrap();
        let word1 = sql.get(&word.name).unwrap();
        assert_eq!(word.message, word1.message);
        sql.delete(&word.name).unwrap();
        
        let err = sql.get("nofound").expect_err("notfound errir");
        match  err {
            InternalError::NotFound => {},
            InternalError::Unknow(unknow_err) => {
              panic!("{}",unknow_err)
            }
        }

        // assert_eq!(sql, None);
        // assert_ne!(sql, None);
    }
}
