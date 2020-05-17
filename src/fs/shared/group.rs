use crate::error::DspfsError;
use crate::fs::shared;
use crate::store::SharedStore;
use crate::user::PublicUser;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Group {
    files: HashMap<PublicUser, HashMap<PathBuf, shared::File>>,

    users: HashSet<PublicUser>,
}

impl Group {
    /// Adds a file to the hashmap of a user. This can be `me`.
    /// TODO: when a file is added, we should probably broadcast that this new file
    ///       exists so others can download it (That's why this fn is async)
    pub async fn add_file(&self, _from: &PublicUser, _file: shared::File) {}

    pub async fn get_my_files(
        &self,
        store: SharedStore,
    ) -> Result<impl Iterator<Item = &shared::File>, DspfsError> {
        let guard = store.read().await;
        let me = guard.get_self_user().as_ref().ok_or_else(|| {
            DspfsError::NotFoundInStore("Group::add_file(): Could not find user in store".into())
        })?;

        Ok(self
            .files
            .get(&me)
            .ok_or_else(|| DspfsError::from("User not found"))?
            .values())
    }

    pub fn list_files_func(&self, path: PathBuf) -> Vec<shared::File> {
        self.files
            .iter()
            .flat_map(|(_, b)| {
                b.iter().filter_map(|(p, f)| {
                    if p.parent() == Some(&path) {
                        return Some(f.clone());
                    }
                    None
                })
            })
            .collect()
    }

    pub fn list_files_imper(&self, path: PathBuf) -> Vec<shared::File> {
        let mut files: Vec<shared::File> = Vec::new();
        for (_, user_files) in self.files.iter() {
            for (bpath, file) in user_files {
                if bpath.parent() == Some(&path) {
                    files.push(file.clone());
                }
            }
        }
        files
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::shared;
    use crate::fs::shared::Group;
    use crate::user::PublicUser;
    use ring::digest::{digest, SHA512};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn simple_list_test() {
        let test_file_a = shared::File {
            fhash: digest(&SHA512, vec![1, 2, 3].as_ref()).into(),
            size: 12,
            path: "/asd/asd".into(),
        };
        let test_file_b = shared::File {
            fhash: digest(&SHA512, vec![1, 2, 3].as_ref()).into(),
            size: 12,
            path: "/def/asd".into(),
        };
        let test_file_c = shared::File {
            fhash: digest(&SHA512, vec![1, 2, 3].as_ref()).into(),
            size: 12,
            path: "/asd/abc".into(),
        };

        let mut paths_a: HashMap<PathBuf, shared::File> = Default::default();
        let mut paths_b: HashMap<PathBuf, shared::File> = Default::default();
        paths_a.insert(test_file_a.path.clone(), test_file_a.clone());
        paths_a.insert(test_file_b.path.clone(), test_file_b);
        paths_b.insert(test_file_c.path.clone(), test_file_c.clone());

        let mut g = Group {
            files: Default::default(),
            users: Default::default(),
        };

        let user_a = PublicUser::new(vec![0].into(), "user_a");
        let user_b = PublicUser::new(vec![1].into(), "user_b");

        g.files.insert(user_a, paths_a);
        g.files.insert(user_b, paths_b);

        let res1 = g.list_files_imper("/asd".into());
        let res2 = g.list_files_func("/asd".into());

        assert!(res1.contains(&test_file_a));
        assert!(res1.contains(&test_file_c));
        assert_eq!(res1.len(), 2);
        assert!(res2.contains(&test_file_a));
        assert!(res2.contains(&test_file_c));
        assert_eq!(res2.len(), 2);
    }
}
