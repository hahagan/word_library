use std::string;


pub struct World {
    pub name: String,
    pub message: String
}

pub trait Store<T,E> {
    fn insert(&self, word: World) -> Result<T,E>;
    fn update(&self, word: World) -> Result<T,E>;
    fn delete(&self, word: &str) -> Result<T,E>;
    fn get(&self, id: &str) -> Result<World,E>;
}