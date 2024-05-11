use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path;

use crate::config;

use super::Note;

pub type NoteIndexContainer = std::rc::Rc<std::cell::RefCell<NoteIndex>>;

/// Contains an indexed and hashed list of notes
pub struct NoteIndex {
    /// The wrapped HashMap, available only in the data module.
    pub(super) inner: HashMap<String, Note>,
}

impl NoteIndex {
    /// Reads a passed directory recursively, returning a hashmap containing
    ///  - An entry for every '.md' file in the directory or any subdirectories
    ///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
    ///  - The value will be an instance of Note containing metadata of the file.
    ///
    /// All files that lead to IO errors when loading are ignored.
    pub fn new(directory: &path::Path, config: &config::Config) -> Self {
        Self {
            inner: walkdir::WalkDir::new(directory)
                .into_iter()
                // Ignore dot-folders and dotfiles
                .filter_entry(is_not_hidden)
                // Check only OKs
                .flatten()
                // Check only markdown files
                .filter(|entry| valid_ending(entry, config))
                // Convert tiles to notes and skip errors
                .flat_map(|entry| Note::from_path(entry.path()))
                // Extract name and convert to id
                .map(|note| (super::name_to_id(&note.name), note))
                // Collect into hash map
                .collect(),
        }
    }

    /// Wrapper of the HashMap::get() Function
    pub fn get(&self, key: &str) -> Option<&Note> {
        self.inner.get(key)
    }

    /// Registers a new note found in the given path in this index.
    pub fn register(&mut self, note_path: &path::Path) {
        if let Ok(note) = Note::from_path(note_path) {
            self.inner.insert(super::name_to_id(&note.name), note);
        }
    }

    /// Removes a note
    pub fn remove(&mut self, id: &str) {
        self.inner.borrow_mut().remove(id);
    }

    /// Returns an iterator over id pairs of notes linked from this note.
    pub fn links_vec(&self, source_id: &str) -> Vec<(String, String)> {
        self.inner
            .get(source_id)
            .map(|source| {
                source
                    .links
                    .iter()
                    .flat_map(|link_id| {
                        self.inner
                            .get(link_id)
                            .map(|note| note.name.clone())
                            .map(|name| (link_id.to_owned(), name))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns an iterator over id pairs of notes linking to this note.
    pub fn blinks_vec(&self, target_id: &str) -> Vec<(String, String)> {
        let id_copy = target_id.to_string();
        self.inner
            .iter()
            .filter(|(_other_id, note)| note.links.contains(&id_copy))
            .map(|(id, note)| (id.to_owned(), note.name.to_owned()))
            .collect()
    }
}
/// Checks if the given dir entry is 'hidden', i.e. not the root of a search and prefixed by a dot.
fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
}
/// Checks if the given dir entry is a valid file, i.e. a file whose name ends one of the endings configured in the config file.
fn valid_ending(entry: &walkdir::DirEntry, config: &config::Config) -> bool {
    entry.file_type().is_file()
        && (config.get_endings().contains(&"*".to_string())
            || match entry.path().extension() {
                Some(ending) => config
                    .get_endings()
                    .contains(&ending.to_string_lossy().to_string()),
                None => config.get_endings().contains(&"".to_string()),
            })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing() {
        let index = NoteIndex::new(
            std::path::Path::new("./tests/common/notes/"),
            &config::Config::default(),
        );

        assert_eq!(index.inner.len(), 11);

        assert!(!index.inner.contains_key("booksold"));

        let os = index.inner.get("operating-systems").unwrap();
        let lg = index.inner.get("lie-group").unwrap();
        let ma = index.inner.get("manifold").unwrap();

        assert_eq!(os.links.len(), 6);
        assert_eq!(os.tags, ["#os"]);
        assert_eq!(os.name, "Operating Systems");
        assert_eq!(os.words, 41);

        assert_eq!(lg.links, ["manifold", "smooth-map", "topology"]);
        assert_eq!(ma.tags.len(), 2);
    }

    #[test]
    fn test_links_blinks() {
        let index = NoteIndex::new(
            std::path::Path::new("./tests/common/notes/"),
            &config::Config::default(),
        );

        assert_eq!(index.inner.len(), 11);

        assert_eq!(
            index.links_vec("lie-group"),
            vec![
                ("manifold".to_string(), "Manifold".to_string()),
                ("smooth-map".to_string(), "Smooth Map".to_string()),
                ("topology".to_string(), "Topology".to_string()),
            ]
        );
        assert_eq!(
            index.blinks_vec("lie-group"),
            vec![("manifold".to_string(), "Manifold".to_string()),]
        );
    }
}