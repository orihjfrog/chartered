//! Generates the Git folder/file tree that's returned back to the user
//! containing the config & crate manifests. Only contains crates that
//! the user has access to.

use crate::git::packfile::high_level::GitRepository;
use arrayvec::ArrayVec;
use chartered_db::crates::Crate;
use std::collections::BTreeMap;

#[derive(serde::Serialize)]
pub struct CrateFileEntry<'a> {
    #[serde(flatten)]
    inner: &'a chartered_types::cargo::CrateVersion<'a>,
    cksum: &'a str,
    yanked: bool,
}

pub struct Tree {
    crates: BTreeMap<String, String>,
}

impl Tree {
    /// Grabs all the crates that the user has access to and writes out the manifests to
    /// `self.crates`.
    pub async fn build(db: chartered_db::ConnectionPool, user_id: i32, org_name: String) -> Self {
        let mut crates = BTreeMap::new();

        for (crate_def, versions) in Crate::list_with_versions(db, user_id, org_name)
            .await
            .unwrap()
        {
            // the manifest we'll be returning to the user
            let mut file = String::new();

            // loop over all versions for the crate, serialising each version to json
            // and writing them to the manifest split by newline.
            for version in versions {
                let cksum = version.checksum.clone();
                let yanked = version.yanked;
                let version = version.into_cargo_format(&crate_def);

                let entry = CrateFileEntry {
                    inner: &version,
                    cksum: &cksum,
                    yanked,
                };

                file.push_str(&serde_json::to_string(&entry).unwrap());
                file.push('\n');
            }

            // insert the crate into `self.crates`
            crates.insert(crate_def.name, file);
        }

        Self { crates }
    }

    /// Writes all the crate manifests from `self.crates` out to the given `GitRepository`.
    pub fn write_to_packfile<'a>(
        &'a self,
        repo: &mut GitRepository<'a>,
    ) -> Result<(), anyhow::Error> {
        for (name, content) in &self.crates {
            let crate_folder = get_crate_folder(name);
            repo.insert(crate_folder, name, content.as_bytes())?;
        }

        Ok(())
    }
}

/// Crates with a total of 1, 2 or 3 characters in the same are written out to directories named
/// 1, 2 or 3 respectively as per the cargo spec. Anything else we'll build out a normal tree for
/// using the frist four characters of the crate name, 2 for the first directory and the other 2
/// for the second.
fn get_crate_folder(crate_name: &str) -> ArrayVec<&str, 2> {
    let mut folders = ArrayVec::new();

    match crate_name.len() {
        0 => {}
        1 => folders.push("1"),
        2 => folders.push("2"),
        3 => {
            folders.push("3");
            folders.push(&crate_name[..1]);
        }
        _ => {
            folders.push(&crate_name[..2]);
            folders.push(&crate_name[2..4]);
        }
    }

    folders
}

#[cfg(test)]
mod test {
    #[test]
    fn get_crate_folder() {
        let folder = super::get_crate_folder("");
        let mut folder = folder.iter();
        assert_eq!(folder.next(), None);

        let folder = super::get_crate_folder("a");
        let mut folder = folder.iter();
        assert_eq!(folder.next(), Some(&"1"));
        assert_eq!(folder.next(), None);

        let folder = super::get_crate_folder("ab");
        let mut folder = folder.iter();
        assert_eq!(folder.next(), Some(&"2"));
        assert_eq!(folder.next(), None);

        let folder = super::get_crate_folder("abc");
        let mut folder = folder.iter();
        assert_eq!(folder.next(), Some(&"3"));
        assert_eq!(folder.next(), Some(&"a"));

        let folder = super::get_crate_folder("abcd");
        let mut folder = folder.iter();
        assert_eq!(folder.next(), Some(&"ab"));
        assert_eq!(folder.next(), Some(&"cd"));
    }
}
