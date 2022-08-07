use std::fmt;
use sqlite::{Connection, Statement, Type, Value};

use crate::store::{Store, World};
const LIBNAME:&str = "library";
const CREATE_TABLE:&str = r#"CREATE TABLE  IF NOT EXISTS library (
      word TEXT PRIMARY KEY,
      message TEXT NOT NULL
    )"#;

extern crate sqlite;

type Error = Box<dyn std::error::Error>;

struct Sqlite {
  connect: sqlite::Connection
}

impl Sqlite {
  fn create_table(&self)  -> Result<(),Error> {
    let con = &self.connect;
    con.execute(CREATE_TABLE)?;
    Ok(())
  }
}


impl  Store<World,Error> for Sqlite {
  fn insert(&self, word: World) -> Result<World,Error> {
    let con = &self.connect;
    let mut stmt = con.prepare("INSTER INTO {} (word, message) VALUES ({}, {})")?
    .bind(0, LIBNAME)?
    .bind(1, word.name.as_str())?
    .bind(2, word.message.as_str())?;

    stmt.next()?;

    Ok(word)
  }
  fn update(&self, word: World) ->  Result<World,Error> {
    return Ok(word);

  }
  fn delete(&self, word: &str) ->  Result<World,Error>{
    Ok(World{
      name: word.to_string(),
      message: LIBNAME.to_string(),
    })
  }
  fn get(&self, id: &str) -> Result<World,Error> {
    return Ok(World{
      name: id.to_string(),
      message: LIBNAME.to_string(),
    });

  }
}

impl Sqlite {
  pub fn new(path:String) -> Result<Sqlite, Box<dyn std::error::Error>> {
    
    let con = Connection::open_with_flags(path, 
      sqlite::OpenFlags::new().set_no_mutex().set_read_write().set_create())?;

    let s = Sqlite{
      connect: con
    };
    s.create_table()?;

    Ok(s)
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sql = Sqlite::new(String::from("qwe")).unwrap();
        println!("{}", sql.connect.change_count())
        // assert_eq!(sql, None);
        // assert_ne!(sql, None);
    }
}
