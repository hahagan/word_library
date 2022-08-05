use std::string;

extern crate sqlite;
struct Sqlite {
  connect: sqlite::Connection
}

impl Sqlite {
  pub fn new(path:String) -> Result<Sqlite, Box<dyn std::error::Error>> {
    // match sqlite::open(path) {
    //   Ok(con) => {
    //     Ok(Sqlite{
    //       connect: con
    //     })
    //   }
    //   Err(e) => Err(e)
        
    // }
    let con = sqlite::open(path)?;
    Ok(Sqlite{
      connect: con
    })
    
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
