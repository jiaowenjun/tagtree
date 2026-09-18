use std::hash::Hash;

use crate::{
    error::{Error, Result},
    path::{normalize_paths, validate_path},
    path_tree::{PathTree, TagNode},
};

/// An item's current tags returned after a subtree mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTags<T> {
    /// The affected item.
    pub item: T,
    /// The item's complete normalized tag set after the mutation.
    pub tags: Vec<String>,
}

/// A read-only summary of a tag tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagTreeSummary {
    /// The root of the owned tree snapshot.
    pub root: TagNode,
    /// Number of paths with directly assigned items, including the root.
    pub populated_path_count: usize,
    /// Number of items assigned directly to the untagged root.
    pub untagged_item_count: usize,
}

/// A hierarchical collection of items and their tag paths.
#[derive(Debug)]
pub struct TagTree<T> {
    tree: PathTree<T>,
}

impl<T: Eq + Hash + Clone> TagTree<T> {
    /// Creates an empty tag index.
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

    /// Returns the number of distinct items in the index.
    pub fn len(&self) -> usize {
        self.tree.item_count()
    }

    /// Returns whether the index contains no items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns whether `item` has tagged or untagged membership in the index.
    pub fn contains_item(&self, item: &T) -> bool {
        self.tree.contains_item(item)
    }

    /// Returns the item's assigned tags, excluding the untagged root.
    pub fn tags_for(&self, item: &T) -> Vec<String> {
        self.tree
            .paths_for(item)
            .into_iter()
            .filter(|path| !path.is_empty())
            .collect()
    }

    /// Builds an owned snapshot and aggregate counts for the complete index.
    pub fn summary(&self) -> TagTreeSummary {
        TagTreeSummary {
            root: self.tree.snapshot(),
            populated_path_count: self.tree.populated_paths().len(),
            untagged_item_count: self.tree.root_item_count(),
        }
    }
}

impl<T: Eq + Hash + Clone + Ord> TagTree<T> {
    /// Returns items assigned to `path` or any descendant in ascending order.
    pub fn items_under(&self, path: &str) -> Result<Vec<T>> {
        validate_path(path)?;
        let mut items = self
            .tree
            .items_under(path)?
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        items.sort_unstable();
        Ok(items)
    }

    /// Moves a complete tag subtree and returns affected items in ascending order.
    pub fn move_subtree(&mut self, old_path: &str, new_path: &str) -> Result<Vec<ItemTags<T>>> {
        validate_path(old_path)?;
        validate_path(new_path)?;
        let affected_items = self.items_under(old_path)?;
        self.tree.move_subtree(old_path, new_path)?;
        if new_path.is_empty() {
            self.remove_redundant_root_assignments(&affected_items);
        }
        Ok(self.assignments(affected_items))
    }

    /// Removes a tag subtree and returns affected items in ascending order.
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

    fn remove_redundant_root_assignments(&mut self, items: &[T]) {
        let root_path = String::new();
        for item in items {
            let paths = self.tree.paths_for(item);
            if paths.len() > 1 && paths.iter().any(String::is_empty) {
                self.tree
                    .remove_from_paths(item, std::slice::from_ref(&root_path));
            }
        }
    }
}

impl<T: Eq + Hash + Clone> Default for TagTree<T> {
    fn default() -> Self {
        Self::new()
    }
}
