use indexmap::IndexMap;

use super::low_level::{
    Commit, CommitUserInfo, HashOutput, PackFileEntry, TreeItem as LowLevelTreeItem, TreeItemKind,
};

#[derive(Default, Debug)]
pub struct Directory(IndexMap<String, Box<TreeItem>>);

impl Directory {
    fn into_packfile_entries<'a>(
        &'a self,
        pack_file: &mut IndexMap<HashOutput, PackFileEntry<'a>>,
    ) -> HashOutput {
        let mut tree = Vec::with_capacity(self.0.len());

        for (name, item) in &self.0 {
            tree.push(match item.as_ref() {
                TreeItem::Blob(hash) => LowLevelTreeItem {
                    kind: TreeItemKind::File,
                    name: &name,
                    hash: *hash,
                },
                TreeItem::Directory(dir) => LowLevelTreeItem {
                    kind: TreeItemKind::Directory,
                    name: &name,
                    hash: dir.into_packfile_entries(pack_file),
                },
            })
        }

        let tree = PackFileEntry::Tree(tree);
        let hash = tree.hash().unwrap();
        pack_file.insert(hash, tree);

        hash
    }
}

#[derive(Debug)]
pub enum TreeItem {
    Blob(HashOutput),
    Directory(Directory),
}

#[derive(Default, Debug)]
pub struct GitRepository<'a> {
    file_entries: IndexMap<HashOutput, PackFileEntry<'a>>,
    tree: Directory,
}

impl<'a> GitRepository<'a> {
    pub fn insert(&mut self, path: Vec<String>, file: String, content: &'a [u8]) {
        let mut directory = &mut self.tree;

        for part in path {
            let tree_item = directory
                .0
                .entry(part)
                .or_insert_with(|| Box::new(TreeItem::Directory(Directory::default())));

            if let TreeItem::Directory(d) = tree_item.as_mut() {
                directory = d;
            } else {
                panic!("one of the path items was a blob");
            }
        }

        let entry = PackFileEntry::Blob(content);

        // todo: handle overwriting error
        let file_hash = entry.hash().unwrap();
        directory
            .0
            .insert(file, Box::new(TreeItem::Blob(file_hash)));
        self.file_entries.insert(file_hash, entry);
    }

    pub fn commit(
        &'a mut self,
        name: &'static str,
        email: &'static str,
        message: &'static str,
    ) -> (HashOutput, Vec<PackFileEntry<'a>>) {
        let tree_hash = self.tree.into_packfile_entries(&mut self.file_entries);

        let commit_user = CommitUserInfo {
            name,
            email,
            time: chrono::Utc::now(),
        };

        let commit = PackFileEntry::Commit(Commit {
            tree: tree_hash,
            author: commit_user,
            committer: commit_user,
            message,
        });

        let commit_hash = commit.hash().unwrap();
        self.file_entries.insert(commit_hash, commit);

        // TODO: make PackFileEntry copy and remove this clone
        (commit_hash, self.file_entries.values().cloned().collect())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let mut x = super::GitRepository::default();
        // x.insert(vec![], "a".to_string(), "nerd".as_ref());
        x.insert(
            vec!["a".to_string(), "b".to_string()],
            "c".to_string(),
            "nerd".as_ref(),
        );
        x.insert(vec![], "b".to_string(), "nerd".as_ref());
        panic!("{:#?}", x);
    }
}
