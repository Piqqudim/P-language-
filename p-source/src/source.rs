use p_lexer::FileId;
use std::path::{Path,PathBuf};

#[derive(Debug,Default)]
pub struct  SourceMap{
    files: Vec<(PathBuf,String)>,
}

impl SourceMap{
    pub fn new() -> Self{ Self { files: Vec::new() }}

    pub fn register(&mut self, path: PathBuf, content:String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push((path,content));
        id
    }

    pub fn path(&self, id: FileId) -> Option<&Path>{
        self.files.get(id.0 as usize).map(|(p,_)| p.as_path())
    }

    pub fn content(&self, id:FileId)->Option<&str>{
        self.files.get(id.0 as usize).map(|(_,s)| s.as_str())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn register_and_lookup_round_trips(){
        let mut map = SourceMap::new();
        let id = map.register(PathBuf::from("main.p"), "page Home\n".to_string());
        assert_eq!(map.path(id), Some(Path::new("main.p")));
        assert_eq!(map.content(id), Some("page Home\n"));
    }

    #[test]
    fn distinct_files_get_distinct_ids(){
        let mut map = SourceMap::new();
        let a = map.register(PathBuf::from("a.p"), "x".into());
        let b = map.register(PathBuf::from("b.p"), "y".into());
        assert_ne!(a,b);
    }
}
