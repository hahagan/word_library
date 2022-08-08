use std::{fmt};

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub message: String
}

pub trait Store<T,E> {
    fn insert(&self, word: &World) -> Result<T,E>;
    fn update(&self, word: &World) -> Result<T,E>;
    fn delete(&self, word: &str) -> Result<T,E>;
    fn get(&self, id: &str) -> Result<World,E>;
}


#[derive(Debug)]
pub struct Error<T>{
    pub message: String,
    pub code: i32,
    pub err: Option<T>
}

impl<T:std::fmt::Debug> std::error::Error for Error<T> {}
impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "{}", self.message)
    }
}



