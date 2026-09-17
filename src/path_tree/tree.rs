use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{Error, Result};

use super::{
    node::{Node, NodeId},
    treeview::TagNode,
};

/// 路径分隔符
const PATH_SEP: &str = "/";
/// 根节点索引
pub(crate) const ROOT_ID: NodeId = 0;

/// 判断 `candidate` 是否为 `ancestor` 的严格后代路径。
/// 与 `find_by_path`/`make_by_path` 一致地忽略空段，保证字符串判定与树结构判定等价。
fn is_descendant_path(candidate: &str, ancestor: &str) -> bool {
    let mut candidate_segments = candidate.split(PATH_SEP).filter(|s| !s.is_empty());
    let mut ancestor_segments = ancestor.split(PATH_SEP).filter(|s| !s.is_empty());

    loop {
        match (ancestor_segments.next(), candidate_segments.next()) {
            (Some(ancestor_seg), Some(candidate_seg)) if ancestor_seg == candidate_seg => {}
            (None, Some(_)) => return true,
            _ => return false,
        }
    }
}

#[derive(Debug)]
/// A tree that maps slash-separated paths to sets of items.
///
/// Each item can be attached to multiple paths. Queries can inspect either the
/// items directly attached to a node or the union of a complete subtree.
///
/// ## Path rules
///
/// - Paths use `/` as the separator, for example `"work/project/urgent"`.
/// - Empty segments are ignored while traversing paths, so `"a//b"` behaves
///   like `"a/b"`.
/// - The empty path `""` addresses the root node.
///
/// ## Item requirements
///
/// `T` must implement `Eq + Hash + Clone` because values are stored in sets and
/// cloned when one item is attached to more than one path.
pub(crate) struct PathTree<T> {
    arena: Vec<Node<T>>,
    /// (ParentID, ChildName) -> ChildID
    child_index: HashMap<(NodeId, String), NodeId>,
}

/// Internal path tree storage for [`crate::TagTree`].
impl<T: Eq + Hash + Clone> PathTree<T> {
    /// Creates an empty path tree with a display label for the root node.
    ///
    /// The root is always addressed by the empty path (`""`); `root_label`
    /// only affects snapshots.
    pub(crate) fn new(root_label: &str) -> Self {
        Self {
            arena: vec![Node::new_root(root_label)],
            child_index: HashMap::new(),
        }
    }

    /// Moves a complete subtree to `new_path`.
    ///
    /// If the destination already exists, the two subtrees are merged.
    ///
    /// # Errors
    ///
    /// Returns an error when `old_path` does not exist, when the root is moved,
    /// or when a subtree is moved into one of its descendants. All checks run
    /// before any node is created, so a rejected move leaves the tree
    /// unchanged.
    pub(crate) fn move_subtree(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        let old_node_id = self
            .find_by_path(old_path)
            .ok_or_else(|| Error::PathNotFound(old_path.to_string()))?;
        if old_node_id == ROOT_ID {
            return Err(Error::CannotMoveRoot);
        }
        if is_descendant_path(new_path, old_path) {
            return Err(Error::CannotMoveIntoDescendant {
                from: old_path.to_string(),
                to: new_path.to_string(),
            });
        }
        let new_node_id = self.make_by_path(new_path);
        self.merge(old_node_id, new_node_id)
    }

    /// Adds an item to one or more paths, creating missing paths as needed.
    pub(crate) fn add_to_paths(&mut self, item: &T, paths: &[String]) {
        for path in paths {
            let node_id = self.make_by_path(path);
            self.get_node_mut(node_id).add_item(item.clone());
        }
    }

    /// Removes an item from the listed paths.
    pub(crate) fn remove_from_paths(&mut self, item: &T, paths: &[String]) {
        for path in paths {
            if let Some(node_id) = self.find_by_path(path) {
                self.get_node_mut(node_id).remove_item(item);
            }
        }
    }

    /// Replaces an item's path assignments.
    pub(crate) fn replace_paths(&mut self, item: &T, old_paths: &[String], new_paths: &[String]) {
        self.remove_from_paths(item, old_paths);
        self.add_to_paths(item, new_paths);
    }

    /// Returns the set union of items attached to `path` and all descendants.
    ///
    /// # Errors
    ///
    /// Returns an error when `path` does not exist.
    pub(crate) fn items_under(&self, path: &str) -> Result<HashSet<&T>> {
        let node_id = self
            .find_by_path(path)
            .ok_or_else(|| Error::PathNotFound(path.to_string()))?;
        Ok(self.get_node_items(node_id))
    }

    /// Returns all paths containing `item`, sorted lexicographically.
    pub(crate) fn paths_for(&self, item: &T) -> Vec<String> {
        let mut paths = Vec::new();
        let mut stack = vec![ROOT_ID];

        while let Some(curr_id) = stack.pop() {
            let node = self.get_node(curr_id);

            if node.contains(item) {
                paths.push(self.get_node_path(curr_id));
            }

            for &child_id in node.iter_children() {
                stack.push(child_id);
            }
        }

        paths.sort();
        paths
    }

    /// Lists paths with at least one item directly attached, sorted
    /// lexicographically.
    pub(crate) fn populated_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        let mut stack = vec![ROOT_ID];

        while let Some(curr_id) = stack.pop() {
            let node = self.get_node(curr_id);
            if !node.is_empty() {
                paths.push(self.get_node_path(curr_id));
            }

            for &child_id in node.iter_children() {
                stack.push(child_id);
            }
        }

        paths.sort();
        paths
    }

    /// Builds an owned read-only snapshot for presentation.
    pub(crate) fn snapshot(&self) -> TagNode {
        TagNode::from_tree(self)
    }

    /// Returns the number of items attached directly to the root.
    pub(crate) fn root_item_count(&self) -> usize {
        self.get_node(ROOT_ID).iter_bag().count()
    }
}

impl<T: Eq + Hash + Clone> Default for PathTree<T> {
    fn default() -> Self {
        Self::new("root")
    }
}

/// 内部 API（只读）
impl<T: Eq + Hash + Clone> PathTree<T> {
    /// 获取节点的不可变引用
    pub(crate) fn get_node(&self, node_id: NodeId) -> &Node<T> {
        &self.arena[node_id]
    }

    /// 获取指定节点的路径字符串
    pub(crate) fn get_node_path(&self, node_id: NodeId) -> String {
        if self.is_root(node_id) {
            return String::new();
        }

        if self.is_root(self.get_parent_id(node_id)) {
            return self.get_node_name(node_id).to_string();
        }

        let mut curr_id = node_id;
        let mut names = Vec::new();

        while !self.is_root(curr_id) {
            names.push(self.get_node_name(curr_id));
            curr_id = self.get_parent_id(curr_id);
        }

        names.reverse();
        names.join(PATH_SEP)
    }

    /// 获取指定节点及其所有子节点中的数据项
    pub(crate) fn get_node_items(&self, node_id: NodeId) -> HashSet<&T> {
        let mut items = HashSet::new();
        let mut stack = vec![node_id];

        while let Some(curr_id) = stack.pop() {
            let node = self.get_node(curr_id);
            items.extend(node.iter_bag());

            for &child_id in node.iter_children() {
                stack.push(child_id);
            }
        }

        items
    }

    pub(crate) fn get_tree_item_counts(&self) -> Vec<usize> {
        let node_count = self.arena.len();
        let mut counts = vec![0; node_count];
        let mut subtree_sets: Vec<Option<HashSet<&T>>> = vec![None; node_count];

        let mut stack = vec![(ROOT_ID, false)];

        while let Some((node_id, visited)) = stack.pop() {
            if !visited {
                stack.push((node_id, true));
                for &child_id in self.get_node(node_id).iter_children() {
                    stack.push((child_id, false));
                }
                continue;
            }

            let node = self.get_node(node_id);
            let mut set: HashSet<&T> = node.iter_bag().collect();

            for &child_id in node.iter_children() {
                let mut child_set = subtree_sets[child_id]
                    .take()
                    .expect("child subtree set should be computed before parent");

                if set.len() < child_set.len() {
                    std::mem::swap(&mut set, &mut child_set);
                }

                set.extend(child_set);
            }

            counts[node_id] = set.len();
            subtree_sets[node_id] = Some(set);
        }

        counts
    }
}

/// 私有 API（只读）
impl<T: Eq + Hash + Clone> PathTree<T> {
    fn get_node_name(&self, node_id: NodeId) -> &str {
        self.get_node(node_id).name()
    }

    fn get_parent_id(&self, node_id: NodeId) -> NodeId {
        self.get_node(node_id).parent_id()
    }

    fn find_child_id(&self, parent_id: NodeId, child_name: &str) -> Option<NodeId> {
        self.child_index
            .get(&(parent_id, child_name.to_string()))
            .copied()
    }

    fn find_by_path(&self, path: &str) -> Option<NodeId> {
        if path.is_empty() {
            return Some(ROOT_ID);
        }

        let mut curr_id = ROOT_ID;
        for name in path.split(PATH_SEP).filter(|s| !s.is_empty()) {
            curr_id = self.find_child_id(curr_id, name)?;
        }
        Some(curr_id)
    }

    fn is_root(&self, node_id: NodeId) -> bool {
        node_id == ROOT_ID
    }

    fn is_ancestor(&self, ancestor_id: NodeId, decendant_id: NodeId) -> bool {
        if ancestor_id == decendant_id {
            return false;
        }
        if self.is_root(ancestor_id) {
            return true;
        }
        if self.is_root(decendant_id) {
            return false;
        }

        let mut curr_id = self.get_parent_id(decendant_id);
        while !self.is_root(curr_id) {
            if curr_id == ancestor_id {
                return true;
            }
            curr_id = self.get_parent_id(curr_id);
        }
        false
    }
}

/// 私有 API
impl<T: Eq + Hash + Clone> PathTree<T> {
    fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node<T> {
        &mut self.arena[node_id]
    }

    fn make_by_path(&mut self, path: &str) -> NodeId {
        if path.is_empty() {
            return ROOT_ID;
        }

        let mut curr_id = ROOT_ID;
        for name in path.split(PATH_SEP).filter(|s| !s.is_empty()) {
            curr_id = self
                .find_child_id(curr_id, name)
                .unwrap_or_else(|| self.create_node(name, curr_id));
        }
        curr_id
    }

    fn alloc_id(&mut self, node: Node<T>) -> NodeId {
        let next_id = self.arena.len();
        self.arena.push(node);
        next_id
    }

    fn create_node(&mut self, name: &str, parent_id: NodeId) -> NodeId {
        let node_id = self.alloc_id(Node::new(name, parent_id));
        self.arena[parent_id].link_child(node_id);
        self.child_index
            .insert((parent_id, name.to_string()), node_id);
        node_id
    }

    fn unlink(&mut self, node_id: NodeId) -> bool {
        if self.is_root(node_id) {
            return false;
        }

        let parent_id = self.get_parent_id(node_id);
        let node_name = self.get_node_name(node_id).to_string();
        self.arena[parent_id].unlink_child(&node_id);
        self.child_index.remove(&(parent_id, node_name));

        true
    }

    fn merge(&mut self, src_id: NodeId, target_id: NodeId) -> Result<()> {
        if src_id == target_id {
            return Ok(());
        }
        if self.is_root(src_id) {
            return Err(Error::CannotMoveRoot);
        }
        if self.is_ancestor(src_id, target_id) {
            return Err(Error::CannotMoveIntoDescendant {
                from: self.get_node_path(src_id),
                to: self.get_node_path(target_id),
            });
        }

        self.unlink(src_id);

        let mut children_to_move = Vec::new();
        let mut children_to_merge = Vec::new();

        for &src_child_id in self.get_node(src_id).iter_children() {
            if let Some(target_child_id) =
                self.find_child_id(target_id, self.get_node_name(src_child_id))
            {
                children_to_merge.push((src_child_id, target_child_id));
            } else {
                children_to_move.push(src_child_id);
            }
        }

        let src_bag = self.get_node_mut(src_id).take_bag();
        self.get_node_mut(target_id).merge_bag(src_bag);

        for src_child_id in children_to_move {
            let child_name = self.get_node_name(src_child_id).to_string();
            self.child_index.remove(&(src_id, child_name.clone()));

            self.get_node_mut(src_child_id).set_parent_id(target_id);
            self.get_node_mut(target_id).link_child(src_child_id);

            self.child_index
                .insert((target_id, child_name), src_child_id);
        }

        for (src_child_id, target_child_id) in children_to_merge {
            self.merge(src_child_id, target_child_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, path_tree::PathTree};

    #[test]
    fn test_new_tree() {
        let tree = PathTree::<usize>::new("tags");
        assert_eq!(tree.populated_paths().len(), 0);
    }

    #[test]
    fn test_default_tree() {
        let tree: PathTree<usize> = PathTree::default();
        assert_eq!(tree.populated_paths().len(), 0);
    }

    #[test]
    fn test_path_root() {
        let tree = PathTree::<usize>::new("tags");
        let root_id = 0;
        assert_eq!(tree.get_node_path(root_id), "");
    }

    #[test]
    fn test_path_single_level() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        let paths = tree.populated_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_path_multi_level() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work/project".to_string()]);

        let paths = tree.populated_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work/project");
    }

    #[test]
    fn test_update_path_success() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        let result = tree.move_subtree("work", "job");
        assert!(result.is_ok());

        let paths = tree.paths_for(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "job");
    }

    #[test]
    fn test_update_path_nonexistent() {
        let mut tree = PathTree::<usize>::new("tags");
        let result = tree.move_subtree("nonexistent", "newpath");
        assert!(result.is_err());
        match result {
            Err(Error::PathNotFound(path)) => {
                assert_eq!(path, "nonexistent");
            }
            _ => panic!("Expected PathNotFound"),
        }
    }

    #[test]
    fn test_update_path_to_ancestor() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work/project".to_string()]);

        let result = tree.move_subtree("work/project", "work");
        assert!(result.is_ok());

        let paths = tree.paths_for(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_update_path_to_descendant() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.add_to_paths(&2, &["work/project".to_string()]);

        let result = tree.move_subtree("work", "work/project");
        assert!(result.is_err());
        match result {
            Err(Error::CannotMoveIntoDescendant { from, to }) => {
                assert_eq!(from, "work");
                assert_eq!(to, "work/project");
            }
            _ => panic!("Expected CannotMoveIntoDescendant"),
        }
    }

    #[test]
    fn test_update_path_to_root() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        let result = tree.move_subtree("work", "");
        assert!(result.is_ok());

        let paths = tree.paths_for(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "");
    }

    #[test]
    fn test_add_to_paths_single_path() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        let items = tree.items_under("work").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_add_to_paths_multiple_paths() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string(), "urgent".to_string()]);

        let work_items = tree.items_under("work").unwrap();
        assert_eq!(work_items.len(), 1);
        assert!(work_items.contains(&1));

        let urgent_items = tree.items_under("urgent").unwrap();
        assert_eq!(urgent_items.len(), 1);
        assert!(urgent_items.contains(&1));
    }

    #[test]
    fn test_add_to_paths_creates_path() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work/project/urgent".to_string()]);

        let paths = tree.populated_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work/project/urgent");

        let items = tree.items_under("work/project/urgent").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_remove_from_paths_single_path() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.remove_from_paths(&1, &["work".to_string()]);

        let items = tree.items_under("work").unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_remove_from_paths_nonexistent_path() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.remove_from_paths(&1, &["nonexistent".to_string()]);

        assert_eq!(tree.populated_paths().len(), 0);
    }

    #[test]
    fn test_items_under_success() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.add_to_paths(&2, &["work".to_string()]);

        let items = tree.items_under("work").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
    }

    #[test]
    fn test_items_under_includes_descendants() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.add_to_paths(&2, &["work/project".to_string()]);

        let items = tree.items_under("work").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
    }

    #[test]
    fn test_items_under_nonexistent_path() {
        let tree = PathTree::<usize>::new("tags");
        let result = tree.items_under("nonexistent");
        assert!(result.is_err());
        match result {
            Err(Error::PathNotFound(path)) => {
                assert_eq!(path, "nonexistent");
            }
            _ => panic!("Expected PathNotFound"),
        }
    }

    #[test]
    fn test_paths_for_single() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        let paths = tree.paths_for(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_paths_for_multiple() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["b".to_string(), "a".to_string(), "c".to_string()]);

        let paths = tree.paths_for(&1);
        assert_eq!(
            paths,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_paths_for_nonexistent() {
        let tree = PathTree::<usize>::new("tags");
        let paths = tree.paths_for(&1);
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_path_normalization_ignores_empty_segments() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["a//b///c".to_string()]);

        let items = tree.items_under("a/b/c").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));

        let paths = tree.populated_paths();
        assert_eq!(paths, vec!["a/b/c".to_string()]);
    }

    #[test]
    fn test_move_subtree_makes_old_path_unreachable() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work//project".to_string()]);

        tree.move_subtree("work", "job").unwrap();

        let old = tree.items_under("work");
        assert!(matches!(old, Err(Error::PathNotFound(_))));

        let items = tree.items_under("job/project").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_move_subtree_recreate_old_path_does_not_alias() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work/task".to_string()]);
        tree.add_to_paths(&2, &["job".to_string()]);

        tree.move_subtree("work", "job").unwrap();

        let old = tree.items_under("work/task");
        assert!(matches!(old, Err(Error::PathNotFound(_))));

        let job_task = tree.items_under("job/task").unwrap();
        assert_eq!(job_task.len(), 1);
        assert!(job_task.contains(&1));

        tree.add_to_paths(&3, &["work/task".to_string()]);

        let job_task = tree.items_under("job/task").unwrap();
        assert_eq!(job_task.len(), 1);
        assert!(job_task.contains(&1));

        let work_task = tree.items_under("work/task").unwrap();
        assert_eq!(work_task.len(), 1);
        assert!(work_task.contains(&3));
    }

    #[test]
    fn test_remove_from_paths_nonexistent_item_is_noop() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.add_to_paths(&2, &["work/project".to_string()]);

        tree.remove_from_paths(&99, &["work".to_string(), "work/project".to_string()]);

        let work = tree.items_under("work").unwrap();
        assert_eq!(work.len(), 2);
        assert!(work.contains(&1));
        assert!(work.contains(&2));
    }

    #[test]
    fn test_merge_existing_paths_combines_items_and_children() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);
        tree.add_to_paths(&2, &["job".to_string()]);
        tree.add_to_paths(&3, &["work/project".to_string()]);
        tree.add_to_paths(&4, &["job/project".to_string()]);

        tree.move_subtree("work", "job").unwrap();

        let job = tree.items_under("job").unwrap();
        assert_eq!(job.len(), 4);
        assert!(job.contains(&1));
        assert!(job.contains(&2));
        assert!(job.contains(&3));
        assert!(job.contains(&4));

        let project = tree.items_under("job/project").unwrap();
        assert_eq!(project.len(), 2);
        assert!(project.contains(&3));
        assert!(project.contains(&4));
    }

    #[test]
    fn test_root_path_listed_when_root_bag_non_empty() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &[String::new()]);

        let paths = tree.populated_paths();
        assert_eq!(paths, vec![String::new()]);
    }

    #[test]
    fn test_paths_for_returns_root_name_for_root_items() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &[String::new()]);

        let paths = tree.paths_for(&1);
        assert_eq!(paths, vec![String::new()]);
    }

    #[test]
    fn test_move_subtree_rejection_creates_no_path() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work".to_string()]);

        assert!(tree.move_subtree("work", "work/sub/deep").is_err());
        assert!(matches!(
            tree.items_under("work/sub/deep"),
            Err(Error::PathNotFound(_))
        ));

        assert!(tree.move_subtree("", "other").is_err());
        assert!(matches!(
            tree.items_under("other"),
            Err(Error::PathNotFound(_))
        ));
    }

    #[test]
    fn test_move_subtree_allows_ancestor_and_self() {
        let mut tree = PathTree::<usize>::new("tags");
        tree.add_to_paths(&1, &["work/project".to_string()]);

        tree.move_subtree("work/project", "work").unwrap();
        assert_eq!(tree.paths_for(&1), vec!["work".to_string()]);

        tree.move_subtree("work", "work").unwrap();
        assert_eq!(tree.paths_for(&1), vec!["work".to_string()]);
    }
}
