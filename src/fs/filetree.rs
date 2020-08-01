use crate::fs::file::File;
use anyhow::{Context, Result};
use std::iter;
use std::path::{Component, Components, Path, PathBuf};
use std::borrow::BorrowMut;
use serde::{Serialize, Deserialize};

/// Normalizes a path (resolves .., .) correctly
/// copied from [cargo](https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61)
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileTree {
    // A directory
    Node {
        name: String,
        children: Vec<FileTree>,
    },
    // A file
    Leaf {
        name: String,
        file: File
    },
}

impl FileTree {
    pub fn new() -> Self {
        FileTree::Node {
            name: Default::default(),
            children: Default::default(),
        }
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a str, &'a File)> + 'a> {
        match self {
            FileTree::Node { children, .. } => {
                Box::new(children.iter().map(|i| i.iter()).flatten())
            }
            FileTree::Leaf{ name, file} => Box::new(iter::once((name.as_ref(), file))),
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a mut String, &'a mut File)> + 'a> {
        match self {
            FileTree::Node { children, .. } => {
                Box::new(children.iter_mut().map(|i| i.iter_mut()).flatten())
            }
            FileTree::Leaf{ name, file} => Box::new(iter::once((name, file))),
        }
    }

    // TODO: Tests
    fn traverse_tree_helper(root: &FileTree, components: &mut Components, mut path: Vec<usize>) -> Result<Vec<usize>> {
        Ok(match &root {
            FileTree::Node { name: _, children } => match components.next() {
                None => path,
                Some(Component::Normal(part)) => {

                    let part = part.to_string_lossy().into_owned();

                    for (index, i) in children.iter().enumerate() {
                        match i {
                            FileTree::Node { name, .. } if name == &part => {
                                path.push(index);
                                return Self::traverse_tree_helper(i, components, path);
                            },
                            FileTree::Leaf { name, .. } if name == &part => {
                                path.push(index);
                                return Ok(path);
                            },
                            _ => (),
                        }
                    }
                    path
                }
                _ => return Err(anyhow::anyhow!("invalid path component"))
            }
            _ => path
        })
    }

    /// Will return the closest matching node of the filetree
    fn traverse_tree(&self, path: impl AsRef<Path>) -> Result<(&FileTree, usize)> {
        let path = Self::traverse_tree_helper(&self, normalize_path(path.as_ref()).components().borrow_mut(), Vec::new())?;

        let len = path.len();

        let mut curr = self;
        for i in path {
            match curr {
                FileTree::Node { name: _, children } => {
                    curr = &children[i];
                }
                _ => unreachable!("This should exist, or otherwise the helper function doesn't work"),
            }
        }

        Ok((curr, len))
    }

    /// The same as [traverse_tree] but it using and returning a mutable [FileTree]
    fn traverse_tree_mut(&mut self, path: impl AsRef<Path>) -> Result<(&mut FileTree, usize)> {
        let path = Self::traverse_tree_helper(&self, normalize_path(path.as_ref()).components().borrow_mut(), Vec::new())?;

        let len = path.len();

        let mut curr = self;
        for i in path {
            match curr {
                FileTree::Node { name: _, children } => {
                    curr = &mut children[i];
                }
                _ => unreachable!("This should exist, or otherwise the helper function doesn't work"),
            }
        }

        Ok((curr, len))
    }

    pub fn find(&self, path: impl AsRef<Path>) -> Option<&FileTree> {
        let path = normalize_path(path.as_ref());

        let (node, len) = self.traverse_tree(path.clone()).ok()?;

        // Did we traverse the entire path
        if path.components().count() != len {
            None
        } else {
            Some(node)
        }
    }

    pub fn find_mut(&mut self, path: impl AsRef<Path>) -> Option<&mut FileTree> {
        let path = normalize_path(path.as_ref());

        let (node, len) = self.traverse_tree_mut(path.clone()).ok()?;

        // Did we traverse the entire path
        if path.components().count() != len {
            None
        } else {
            Some(node)
        }
    }

    /// Deletes a node from the tree and returns it.
    pub fn delete(&mut self, path: impl AsRef<Path>, recursive: bool) -> Option<FileTree> {
        let path = normalize_path(path.as_ref());
        let filename = path.file_name()?.to_string_lossy().into_owned();

        let (node, len) = if let Some(path) = path.parent() {
            self.traverse_tree_mut(path).ok()? // TODO: Error?
        } else {
            (self, 1)
        };

        // Did we traverse the entire path
        if path.components().count() != len + 1 {
            return None
        };

        match node {
            FileTree::Node {name: _, children} => {
                let mut found: Option<usize> = None;
                for (index, child) in children.iter().enumerate() {

                    match child {
                        FileTree::Leaf {name, file: _} => {
                            if name == &filename {
                                found = Some(index);
                            }
                        }
                        FileTree::Node {name, children: _} => {
                            if recursive && name == &filename {
                                found = Some(index);
                            }
                        }
                    }
                }

                found.map(|index| children.remove(index))
            }
            _ => None
        }
    }


    /// Inserts a file into the filetree, with the `path` being a sequence of folders
    /// from te root of the filetree.
    ///
    /// ```
    /// let mut filetree = FileTree::Node {
    ///     name: "".into(),
    ///     children: vec![
    ///         FileTree::Node {
    ///             name: "test".into(),
    ///             children: vec![
    ///                 FileTree::Leaf(Default::default()),
    ///                 // 1. INSERT HERE
    ///             ],
    ///         },
    ///         // 2. INSERT HERE
    ///     ],
    /// };
    ///
    /// // In the `test` subfolder
    /// filetree.insert("test/file.txt", Default::default())
    /// // In the root folder
    /// filetree.insert("file.txt", Default::default())
    ///
    /// ```
    ///
    /// The `path` argument must be relative (may not start with a `/`),
    /// may not contain `..` (go up directories) or `.` (current dir), and may not contain
    /// windows drive letters (how the fuck did you manage that, dspfs doesn't even run on windows)
    pub fn insert(&mut self, path: impl AsRef<Path>, file: File) -> Result<()> {
        let path = normalize_path(path.as_ref());

        let (mut node, len) = if let Some(path) = path.parent() {
            self.traverse_tree_mut(path).context("finding closest filetree node went wrong")?
        } else {
            (self, 1)
        };

        let filename = path.file_name()
            .context("Somehow no filename was found in that path (shouldn't happen) (if it does happen you know either Victor or Jonathan did an oopsie woopsie) (aka you're fucked)")?
            .to_string_lossy()
            .into_owned();

        if let Some(path) = path.parent() {
            for c in path.components().skip(len) {
                let folder = FileTree::Node {
                    children: Vec::new(),
                    name: c.as_os_str().to_string_lossy().into_owned()
                };
                if let FileTree::Node { name: _, children} = node {
                    children.push(folder);
                    node = children.last_mut().unwrap();
                }
            }
        }

        if let FileTree::Node { name: _, children} = node {
            let leaf = FileTree::Leaf{ name: filename,  file };

            children.push(leaf);
        } else {
            return Err(anyhow::anyhow!("path pointed to file, can't insert a file in a non directory."))
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::filetree::FileTree::{Leaf, Node};

    #[test]
    pub fn test_insert() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet/yeet.txt", file.clone()).unwrap();

        assert_eq!(
            f,
            Node {
                name: "".into(),
                children: vec![Node {
                    name: "yeet".into(),
                    children: vec![Leaf{name: "yeet.txt".into(), file},],
                },],
            }
        );
    }

    #[test]
    pub fn test_find() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet/yeet.txt", file.clone()).unwrap();

        let found = f.find("yeet/yeet.txt").unwrap();
        assert_eq!(
            found,
            &Leaf{name: "yeet.txt".into(), file}
        );
    }

    #[test]
    pub fn test_delete() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet/yeet.txt", file.clone()).unwrap();
        let node = f.delete("yeet/yeet.txt", false).unwrap();
        if let Leaf {name: fname, file: ffile} = node {
            assert_eq!(fname, "yeet.txt");
            assert_eq!(ffile, file);
        } else {
            unreachable!()
        }

        assert_eq!(
            f,
            Node {
                name: "".into(),
                children: vec![Node {
                    name: "yeet".into(),
                    children: vec![],
                },],
            }
        );
    }

    #[test]
    pub fn test_delete_recursive() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet/yeet.txt", file).unwrap();
        f.delete("yeet", true).unwrap();

        assert_eq!(
            f,
            Node {
                name: "".into(),
                children: vec![]
            }
        );
    }

    #[test]
    pub fn test_traverse_leaf() {
        let mut f = FileTree::new();

        let file = File::new_empty("test.txt".into());

        f.insert("test.txt", file.clone()).unwrap();

        let result = f.traverse_tree("test.txt").unwrap();

        assert_eq!(result.1, 1);
        assert_eq!(result.0, &Leaf {
            name: "test.txt".to_string(),
            file
        })
    }

    #[test]
    pub fn test_traverse() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet.txt", file.clone()).unwrap();
        f.insert("yeet/yote.txt", file.clone()).unwrap();
        f.insert("yeet/yote/yoinks.txt", file).unwrap();

        let (n, _) = f.traverse_tree("yeet/yote").unwrap();

        match n {
            FileTree::Node { name, .. } => assert_eq!(name, "yote"),
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_find_mut() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet.txt", file.clone()).unwrap();
        f.insert("yeet/yote.txt", file.clone()).unwrap();
        f.insert("yeet/yote/yoinks.txt", file).unwrap();

        let (n, _): (&mut FileTree, _) = f.traverse_tree_mut("yeet/yote").unwrap();

        match n {
            FileTree::Node { name, .. } => assert_eq!(name, "yote"),
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_iter_simple() {
        let file = File::new_empty("yeet.txt".into());

        let t = Node {
            name: "".into(),
            children: vec![Node {
                name: "yeet".into(),
                children: vec![Leaf{name: "yeet.txt".into(), file: file.clone()}],
            }],
        };

        assert_eq!(vec![&file], t.iter().map(|(_, i)| i).collect::<Vec<&File>>())
    }

    #[test]
    pub fn test_insert_nested() {
        let mut f = FileTree::new();

        let file = File::new_empty("yeet.txt".into());

        f.insert("yeet/yeet/yeet.txt", file.clone()).unwrap();

        assert_eq!(
            f,
            Node {
                name: "".into(),
                children: vec![Node {
                    name: "yeet".into(),
                    children: vec![Node {
                        name: "yeet".into(),
                        children: vec![Leaf{name: "yeet.txt".into(), file}],
                    },],
                },],
            }
        );
    }

    #[test]
    pub fn test_insert_nested_partial_match() {
        let mut f = FileTree::new();

        let file = File::new_empty("None".into());

        f.insert("yeet/yote/yeet.txt", file.clone()).unwrap();
        f.insert("yeet/yeet/yeet.txt", file.clone()).unwrap();

        assert_eq!(
            f,
            Node {
                name: "".into(),
                children: vec![Node {
                    name: "yeet".into(),
                    children: vec![
                        Node {
                            name: "yote".into(),
                            children: vec![FileTree::Leaf{name: "yeet.txt".into(), file: file.clone()}],
                        },
                        Node {
                            name: "yeet".into(),
                            children: vec![FileTree::Leaf{name: "yeet.txt".into(), file}],
                        },
                    ],
                },],
            }
        );
    }

    #[test]
    pub fn test_non_canon() {
        let mut f = FileTree::new();
        let file = File::new_empty("yeet.txt".into());

        // Absolute root is not allowed
        assert!(f.insert("/yeet.txt", file.clone()).is_err());
        assert!(f.insert("/../yeet.txt", file.clone()).is_err());

        // This will insert a file in the root of the filetree
        assert!(f.insert("../yeet.txt", file.clone()).is_ok());
        assert!(f.insert("yeet/../yeet2.txt", file.clone()).is_ok());
        assert!(f.insert("./yeet3.txt", file.clone()).is_ok());

        // This will insert a file in yeet/yeet.txt
        assert!(f.insert("yeet/./yeet.txt", file.clone()).is_ok());

        let expected = Node {
            name: "".to_string(),
            children: vec![
                Leaf {
                    name: "yeet.txt".to_string(),
                    file: file.clone(),
                },
                Leaf {
                    name: "yeet2.txt".to_string(),
                    file: file.clone(),
                },
                Leaf {
                    name: "yeet3.txt".to_string(),
                    file: file.clone(),
                },
                Node {
                    name: "yeet".to_string(),
                    children: vec![
                        Leaf {
                            name: "yeet.txt".to_string(),
                            file
                        },
                    ],
                },
            ],
        };

        assert_eq!(expected, f);
    }

    #[test]
    pub fn test_leaf_err() {
        let file = File::new_empty("None".into());
        let mut f = FileTree::Leaf{name: "yeet.txt".into(), file: file.clone()};
        assert!(f.insert("some path", file).is_err());
    }
}

