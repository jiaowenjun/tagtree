use std::{hash::Hash, iter};

use crate::treebag::{Tree, TreeResult, TreeView};

/// An item's persisted tag assignment after a path mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagAssignment<T> {
    pub item: T,
    pub tags: Vec<String>,
}

/// A read-only summary of a tag index.
pub struct TagIndexSummary {
    pub tree: TreeView,
    /// Number of paths with directly assigned items, including the untagged root.
    pub assigned_path_count: usize,
    pub untagged_count: usize,
}

/// A hierarchical, multi-tag index that owns untagged-item semantics.
#[derive(Debug)]
pub struct TagIndex<T> {
    tree: Tree<T>,
}

impl<T: Eq + Hash + Clone> TagIndex<T> {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(""),
        }
    }

    /// Inserts an item or replaces all of its existing tags.
    pub fn upsert(&mut self, item: &T, tags: &[String]) {
        let old_paths = self.tree.get_item_paths(item);
        let new_paths = paths_for_storage(tags);

        if old_paths == new_paths {
            return;
        }

        self.tree.move_item(item, &old_paths, &new_paths);
    }

    /// Removes an item without requiring callers to provide its old tags.
    pub fn remove(&mut self, item: &T) {
        let old_paths = self.tree.get_item_paths(item);
        self.tree.delete_item(item, &old_paths);
    }

    /// Returns persisted tags without exposing the internal untagged root path.
    pub fn tags(&self, item: &T) -> Vec<String> {
        self.tree
            .get_item_paths(item)
            .into_iter()
            .filter(|path| !path.is_empty())
            .collect()
    }

    pub fn summary(&self) -> TagIndexSummary {
        TagIndexSummary {
            tree: self.tree.view(),
            assigned_path_count: self.tree.list_paths().len(),
            untagged_count: self.tree.root_bag_count(),
        }
    }
}

impl<T: Eq + Hash + Clone + Ord> TagIndex<T> {
    /// Returns items assigned to a path or any descendant in descending order.
    pub fn items(&self, path: &str) -> TreeResult<Vec<T>> {
        let mut items = self
            .tree
            .get_items(path)?
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        items.sort_unstable_by(|left, right| right.cmp(left));
        Ok(items)
    }

    /// Renames or merges a path and returns assignments for affected items.
    pub fn rename_path(
        &mut self,
        old_path: &str,
        new_path: &str,
    ) -> TreeResult<Vec<TagAssignment<T>>> {
        let affected_items = self.items(old_path)?;
        self.tree.move_path(old_path, new_path)?;
        Ok(self.assignments(affected_items))
    }

    /// Removes a path subtree while preserving unrelated tags.
    pub fn delete_path(&mut self, path: &str) -> TreeResult<Vec<TagAssignment<T>>> {
        let affected_items = self.items(path)?;
        let descendant_prefix = format!("{path}/");

        for item in &affected_items {
            let old_paths = self.tree.get_item_paths(item);
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

            self.tree.move_item(item, &old_paths, &next_paths);
        }

        Ok(self.assignments(affected_items))
    }

    fn assignments(&self, items: Vec<T>) -> Vec<TagAssignment<T>> {
        items
            .into_iter()
            .map(|item| TagAssignment {
                tags: self.tags(&item),
                item,
            })
            .collect()
    }
}

impl<T: Eq + Hash + Clone> Default for TagIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn paths_for_storage(tags: &[String]) -> Vec<String> {
    if tags.is_empty() {
        return iter::once(String::new()).collect();
    }

    let mut paths = tags.to_vec();
    paths.sort();
    paths.dedup();
    paths
}
