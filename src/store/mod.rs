pub mod gui;

use std::{collections::HashMap, fmt};

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub message: String,
}

pub trait Transaction<T, E>: Write<T, E> + Read<T, E> {
    fn rollback(self) -> Result<(), E>;
    fn commit(self) -> Result<(), E>;
}

pub trait Write<T, E> {
    fn insert(&self, word: &World) -> Result<Option<T>, E>;
    fn update(&self, word: &World) -> Result<Option<T>, E>;
    fn delete(&self, word: &str) -> Result<Option<T>, E>;
}

pub trait Read<T, E> {
    fn get(&self, id: &str) -> Result<World, E>;
    fn list(&self, size: i64) -> Result<Vec<World>, E>;
}
pub trait Store<T, E, F: Transaction<T, E>>: Write<T, E> + Read<T, E> {
    fn begin(&self) -> Result<F, E>;
}

#[derive(Debug)]
pub struct Error<E> {
    pub message: String,
    pub code: i32,
    pub err: Option<E>,
}

impl<T: std::fmt::Debug + fmt::Display> std::error::Error for Error<T> {}
impl<T: fmt::Display> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.err {
            Some(err) => {
                write!(
                    f,
                    "msg: {}, code: {}, raw: {}",
                    self.message, self.code, err
                )
            }
            None => {
                write!(f, "msg: {}, code: {}", self.message, self.code)
            }
        }
    }
}

#[derive(Debug)]
pub enum InternalError<E> {
    NotFound,
    StoreNotFound,
    Unknow(Error<E>),
}

impl<E: std::fmt::Debug + fmt::Display> std::error::Error for InternalError<E> {}
impl<E: fmt::Display> fmt::Display for InternalError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::NotFound => {
                return write!(f, "internal error for item not found");
            }
            InternalError::StoreNotFound => {
                return write!(f, "internal error for item not found");
            }
            InternalError::Unknow(err) => {
                return write!(f, "internal error for unknow {}", err);
            }
        }
        // write!(f, "internal error")
    }
}

// pub struct WordLibrary<T, E, F: Transaction<T, InternalError<E>>, S: Store<T, InternalError<E>, F>>
pub struct WordLibrary<T, E, F>
where
    F: Transaction<T, InternalError<E>>,
    // S: Store<T, InternalError<E>, F>,
{
    stores: HashMap<String, Box<dyn Store<T, InternalError<E>, F>>>,
    // stores: HashMap<String, S>,
}

impl<
        T,
        E: std::fmt::Debug,
        F: Transaction<T, InternalError<E>>,
        // S: Store<T, InternalError<E>, F>,
    > WordLibrary<T, E, F>
{
    pub fn insert(&self, word: &World, key: &str) -> Result<Option<T>, InternalError<E>> {
        let s = self.get_store(key)?;
        s.insert(word)
    }

    pub fn update(&self, word: &World, key: &str) -> Result<Option<T>, InternalError<E>> {
        let s = self.get_store(key)?;
        s.update(word)
    }
    pub fn delete(&self, name: &str, key: &str) -> Result<Option<T>, InternalError<E>> {
        let s = self.get_store(key)?;
        s.delete(name)
    }

    pub fn get(&self, name: &str, key: &str) -> Result<World, InternalError<E>> {
        match self.stores.get(key) {
            Some(s) => s.get(name),
            None => Err(InternalError::StoreNotFound),
        }
    }

    pub fn list(&self, size: i64, key: &str) -> Result<Vec<World>, InternalError<E>> {
        match self.stores.get(key) {
            Some(s) => s.list(size),
            None => Err(InternalError::StoreNotFound),
        }
    }

    fn get_store(
        &self,
        key: &str,
    ) -> Result<&Box<dyn Store<T, InternalError<E>, F>>, InternalError<E>> {
        self.stores.get(key).ok_or(InternalError::StoreNotFound)
    }

    pub fn move_to(&self, name: &str, src: &str, dst: &str) -> Result<Option<T>, InternalError<E>> {
        let s = self.get_store(src)?;
        let d = self.get_store(dst)?;

        let t0 = s.begin()?;

        let word: World;
        match t0.get(name) {
            Ok(w) => word = w,
            Err(err) => {
                t0.rollback().unwrap();
                return Err(err);
            }
        }

        let t1 = d.begin()?;

        if let Err(err) = t1.insert(&word) {
            t0.rollback().unwrap();
            t1.rollback().unwrap();
            return Err(err);
        }

        if let Err(err) = t0.delete(&word.name) {
            t0.rollback().unwrap();
            t1.rollback().unwrap();
            return Err(err);
        }

        // if the commit for insert error, recover before this func
        if let Err(err) = t1.commit() {
            t0.rollback().unwrap();
            return Err(err);
        }

        // if the commit for delete error, the word will occur in two lirabry
        if let Err(err) = t0.commit() {
            return Err(err);
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::store;
    extern crate sqlite;

    fn new_word_library() -> WordLibrary<World, sqlite::Error, store::Tansaction> {
        let k0 = "target/test.sql";
        let k1 = "target/test1.sql";
        let sql0 = store::Sqlite::new(k0.to_owned()).unwrap();
        let sql1 = store::Sqlite::new(k1.to_owned()).unwrap();

        let mut res = WordLibrary {
            stores: HashMap::new(),
        };

        res.stores.insert(k0.to_owned(), Box::new(sql0));
        res.stores.insert(k1.to_owned(), Box::new(sql1));
        return res;
    }

    // #[test]
    fn clean_sqlite() {
        let k0 = "target/test.sql";
        let k1 = "target/test1.sql";
        std::fs::remove_file(k0).unwrap();
        std::fs::remove_file(k1).unwrap();
    }

    #[test]
    fn test_move() {
        let k0 = "target/test.sql";
        let k1 = "target/test1.sql";
        let wd = new_word_library();
        let word = World {
            name: String::from("test"),
            message: String::from("test"),
        };
        if let Err(err) = wd.get_store("notstore") {
            match err {
                InternalError::StoreNotFound => {}
                err => {
                    panic!("{}", err)
                }
            }
        } else {
            panic!("should reuturn store not found error")
        }
        wd.insert(&word, k0).unwrap();
        wd.move_to(&word.name, k0, k1).unwrap();
        wd.get(&word.name, k1).unwrap();
        wd.list(1, k1).unwrap();
        wd.update(&word, k1).unwrap();
        wd.delete(&word.name, k1).unwrap();

        // clean_sqlite()
    }
}
