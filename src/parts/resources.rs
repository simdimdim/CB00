use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Resource {
    id:   u32,
    path: PathBuf,
}
#[derive(Clone, Debug)]
pub struct ResourceManager {
    res: HashMap<String, Vec<Resource>>,
}
