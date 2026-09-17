use std::hash::Hash;

use crate::{
    error::{Error, Result},
    path::{normalize_paths, validate_path},
    path_tree::{PathTree, TagNode},
};

/// An item's current tags returned after a subtree mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTags<T> {
    pub item: T,
    pub tags: Vec<String>,
}

/// A read-only summary of a tag tree.
#[derive(Debug)]
pub struct TagTreeSummary {
    pub root: TagNode,
    /// Number of paths with directly assigned items, including the root.
    pub populated_path_count: usize,
    pub untagged_item_count: usize,
}

/// A hierarchical collection of items and their tag paths.
#[derive(Debug)]
pub struct TagTree<T> {
    tree: PathTree<T>,
}

impl<T: Eq + Hash + Clone> TagTree<T> {
    pub fn new() -> Self {
        Self {
            tree: PathTree::new(""),
        }
    }

    /// Replaces all tags for `item` after normalizing and validating them.
    pub fn set_tags<I, S>(&mut self, item: &T, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut new_paths = normalize_paths(tags)?;
        new_paths.sort();
        let new_paths = if new_paths.is_empty() {
            vec![String::new()]
        } else {
            new_paths
        };
        let old_paths = self.tree.paths_for(item);

        if old_paths == new_paths {
            return Ok(());
        }

        self.tree.replace_paths(item, &old_paths, &new_paths);
        Ok(())
    }

    /// Removes an item and all of its tag assignments.
    pub fn remove_item(&mut self, item: &T) {
        let old_paths = self.tree.paths_for(item);
        self.tree.remove_from_paths(item, &old_paths);
    }

    /// Returns the item's assigned tags, excluding the untagged root.
    pub fn tags_for(&self, item: &T) -> Vec<String> {
        self.tree
            .paths_for(item)
            .into_iter()
            .filter(|path| !path.is_empty())
            .collect()
    }

    pub fn summary(&self) -> TagTreeSummary {
        TagTreeSummary {
            root: self.tree.snapshot(),
            populated_path_count: self.tree.populated_paths().len(),
            untagged_item_count: self.tree.root_item_count(),
        }
    }
}

impl<T: Eq + Hash + Clone + Ord> TagTree<T> {
    /// Returns items assigned to `path` or any descendant in descending order.
    pub fn items_under(&self, path: &str) -> Result<Vec<T>> {
        validate_path(path)?;
        let mut items = self
            .tree
            .items_under(path)?
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        items.sort_unstable_by(|left, right| right.cmp(left));
        Ok(items)
    }

    /// Moves a complete tag subtree and returns the resulting tags for affected items.
    pub fn move_subtree(&mut self, old_path: &str, new_path: &str) -> Result<Vec<ItemTags<T>>> {
        validate_path(old_path)?;
        validate_path(new_path)?;
        let affected_items = self.items_under(old_path)?;
        self.tree.move_subtree(old_path, new_path)?;
        Ok(self.assignments(affected_items))
    }

    /// Removes a tag subtree while preserving unrelated tags.
    ///
    /// The root path `""` is rejected because untagged items live there and
    /// removing the root subtree is ambiguous.
    pub fn remove_subtree(&mut self, path: &str) -> Result<Vec<ItemTags<T>>> {
        validate_path(path)?;
        if path.is_empty() {
            return Err(Error::CannotRemoveRoot);
        }
        let affected_items = self.items_under(path)?;
        let descendant_prefix = format!("{path}/");

        for item in &affected_items {
            let old_paths = self.tree.paths_for(item);
            let retained_paths = old_paths
                .iter()
                .filter(|item_path| {
                    item_path.as_str() != path && !item_path.starts_with(&descendant_prefix)
                })
                .cloned()
                .collect::<Vec<_>>();
            let next_paths = if retained_paths.is_empty() {
                vec![String::new()]
            } else {
                retained_paths
            };

            self.tree.replace_paths(item, &old_paths, &next_paths);
        }

        Ok(self.assignments(affected_items))
    }

    fn assignments(&self, items: Vec<T>) -> Vec<ItemTags<T>> {
        items
            .into_iter()
            .map(|item| ItemTags {
                tags: self.tags_for(&item),
                item,
            })
            .collect()
    }
}

impl<T: Eq + Hash + Clone> Default for TagTree<T> {
    fn default() -> Self {
        Self::new()
    }
}
