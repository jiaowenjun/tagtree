use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use super::{
    error::{TreeError, TreeResult},
    node::{Node, NodeId},
    treeview::TreeView,
};

/// 路径分隔符
const PATH_SEP: &str = "/";
/// 根节点索引
pub(crate) const ROOT_ID: NodeId = 0;

#[derive(Debug)]
/// 一棵用于管理“分层路径 -> 数据项”的树。
///
/// `Tree` 主要面向类似“标签/分类”的场景：每个节点对应一个路径片段（如 `"work"`、`"project"`），
/// 数据项（`T`）可以挂载在任意节点上；同一个数据项也可以同时关联到多个路径。
///
/// ## 路径规则
///
/// - 路径使用 `/` 作为分隔符，例如 `"work/project/urgent"`。
/// - 路径会按 `/` 拆分，并忽略空片段（例如 `"a//b"` 等价于 `"a/b"`）。
/// - 空路径 `""` 表示根节点；根节点的路径字符串为 [`Tree::new`] 的 `root_name`。
///
/// ## 数据项约束
///
/// 当前公开 API 需要 `T: Eq + Hash + Clone`：
/// - 通过 [`Tree::add_item`] 添加时会对 `T` 做 `clone`（因为同一项可能被挂到多个路径上）。
/// - 每个节点内部用集合语义存储数据项：重复添加同一项不会产生重复记录。
pub struct Tree<T> {
    arena: Vec<Node<T>>,
    /// (ParentID, ChildName) -> ChildID
    child_index: HashMap<(NodeId, String), NodeId>,
}

/// 公开 API
impl<T: Eq + Hash + Clone> Tree<T> {
    /// 创建一棵新的树，并设置根节点名称。
    ///
    /// `root_name` 仅影响根节点的显示/路径字符串表示：
    /// - 根节点可用空路径 `""` 访问；
    /// - 根节点的路径字符串为 `root_name`。
    pub fn new(root_name: &str) -> Self {
        Self {
            arena: vec![Node::new_root(root_name)],
            child_index: HashMap::new(),
        }
    }

    /// 将 `old_path` 对应的节点（含其子树）移动/合并到 `new_path`。
    ///
    /// 如果 `new_path` 不存在，会自动创建；若 `new_path` 已存在，则会执行“合并”：
    /// - 把旧节点的所有数据项并入新节点；
    /// - 对同名子节点递归合并；
    /// - 其余子节点整体迁移到新节点名下。
    ///
    /// # Errors
    ///
    /// - 当 `old_path` 不存在时返回 [`TreeError::PathError`]。
    /// - 不能把根节点合并到其它节点。
    /// - 不能把一个节点合并到它的后代节点上。
    pub fn move_path(&mut self, old_path: &str, new_path: &str) -> TreeResult<()> {
        let old_node_id = self
            .find_by_path(old_path)
            .ok_or(TreeError::PathError(format!("路径 {old_path} 不存在")))?;
        let new_node_id = self.make_by_path(new_path);
        self.merge(old_node_id, new_node_id)
    }

    /// 将 `item` 关联到多个路径上（路径不存在会被自动创建）。
    ///
    /// 同一数据项在同一路径下重复添加不会产生重复记录。
    pub fn add_item(&mut self, item: &T, paths: &[String]) {
        for path in paths {
            let node_id = self.make_by_path(path);
            self.get_node_mut(node_id).add_item(item.clone());
        }
    }

    /// 从多个路径中删除数据项的关联关系。
    ///
    /// - 若路径不存在会被忽略（不会报错）。
    /// - 若该路径下不存在该数据项也不会报错。
    pub fn delete_item(&mut self, item: &T, paths: &[String]) {
        for path in paths {
            if let Some(node_id) = self.find_by_path(path) {
                self.get_node_mut(node_id).remove_item(item);
            }
        }
    }

    /// 更新数据项的路径：从 `old_paths` 删除，并添加到 `new_paths`。
    ///
    /// 该操作等价于依次调用 [`Tree::delete_item`] 与 [`Tree::add_item`]。
    pub fn move_item(&mut self, item: &T, old_paths: &[String], new_paths: &[String]) {
        self.delete_item(item, old_paths);
        self.add_item(item, new_paths);
    }

    /// 获取 `path` 下的所有数据项（包含其所有子节点的集合并集）。
    ///
    /// 返回值是对树内部数据项的引用集合，生命周期与 `&self` 绑定。
    ///
    /// # Errors
    ///
    /// 当 `path` 不存在时返回 [`TreeError::PathError`]。
    pub fn get_items(&self, path: &str) -> TreeResult<HashSet<&T>> {
        let node_id = self
            .find_by_path(path)
            .ok_or(TreeError::PathError(format!("路径 {path} 不存在")))?;
        Ok(self.get_node_items(node_id))
    }

    /// 获取 `path` 对应节点自身背包中的数据项（不包含子节点）。
    ///
    /// 若需子树并集，请使用 [`Tree::get_items`]。
    ///
    /// # Errors
    ///
    /// 当 `path` 不存在时返回 [`TreeError::PathError`]。
    pub fn get_bag(&self, path: &str) -> TreeResult<HashSet<&T>> {
        let node_id = self
            .find_by_path(path)
            .ok_or(TreeError::PathError(format!("路径 {path} 不存在")))?;
        Ok(self.get_node(node_id).iter_bag().collect())
    }

    /// 获取包含指定数据项的所有路径（按字典序排序）。
    ///
    /// 如果数据项存在于根节点，则会包含根节点路径（即 `root_name`）。
    pub fn get_item_paths(&self, item: &T) -> Vec<String> {
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

    /// 列出所有“非空节点”的路径（按字典序排序）。
    ///
    /// 这里的“非空”指该节点自身直接挂载了至少一个数据项；
    /// 不包含仅因子节点非空而自身为空的节点。
    pub fn list_paths(&self) -> Vec<String> {
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

    /// 构造一个用于展示的 [`TreeView`]。
    pub fn view(&self) -> TreeView {
        TreeView::from_tree(self)
    }

    /// 获取根节点自身背包中的数据项数量（不包含子节点）。
    pub fn root_bag_count(&self) -> usize {
        self.get_node(ROOT_ID).iter_bag().count()
    }
}

impl<T: Eq + Hash + Clone> Default for Tree<T> {
    fn default() -> Self {
        Self::new("root")
    }
}

/// 内部 API（只读）
impl<T: Eq + Hash + Clone> Tree<T> {
    /// 获取节点的不可变引用
    pub(crate) fn get_node(&self, node_id: NodeId) -> &Node<T> {
        &self.arena[node_id]
    }

    /// 获取指定节点的路径字符串
    pub(crate) fn get_node_path(&self, node_id: NodeId) -> String {
        if self.is_root(node_id) || self.is_root(self.get_parent_id(node_id)) {
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
impl<T: Eq + Hash + Clone> Tree<T> {
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
impl<T: Eq + Hash + Clone> Tree<T> {
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

    fn merge(&mut self, src_id: NodeId, target_id: NodeId) -> TreeResult<()> {
        if src_id == target_id {
            return Ok(());
        }
        if self.is_root(src_id) {
            return Err(TreeError::MergeError("不能合并根节点".to_string()));
        }
        if self.is_ancestor(src_id, target_id) {
            return Err(TreeError::MergeError("不能合并到后代".to_string()));
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
    use super::*;
    use crate::treebag::TreeError;

    #[test]
    fn test_new_tree() {
        let tree = Tree::<usize>::new("tags");
        assert_eq!(tree.list_paths().len(), 0);
    }

    #[test]
    fn test_default_tree() {
        let tree: Tree<usize> = Tree::default();
        assert_eq!(tree.list_paths().len(), 0);
    }

    #[test]
    fn test_path_root() {
        let tree = Tree::<usize>::new("tags");
        let root_id = 0;
        assert_eq!(tree.get_node_path(root_id), "tags");
    }

    #[test]
    fn test_path_single_level() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);

        let paths = tree.list_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_path_multi_level() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work/project".to_string()]);

        let paths = tree.list_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work/project");
    }

    #[test]
    fn test_update_path_success() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);

        let result = tree.move_path("work", "job");
        assert!(result.is_ok());

        let paths = tree.get_item_paths(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "job");
    }

    #[test]
    fn test_update_path_nonexistent() {
        let mut tree = Tree::<usize>::new("tags");
        let result = tree.move_path("nonexistent", "newpath");
        assert!(result.is_err());
        match result {
            Err(TreeError::PathError(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected PathError"),
        }
    }

    #[test]
    fn test_update_path_to_ancestor() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work/project".to_string()]);

        let result = tree.move_path("work/project", "work");
        assert!(result.is_ok());

        let paths = tree.get_item_paths(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_update_path_to_descendant() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);

        let result = tree.move_path("work", "work/project");
        assert!(result.is_err());
        match result {
            Err(TreeError::MergeError(msg)) => {
                assert!(msg.contains("后代"));
            }
            _ => panic!("Expected MergeError"),
        }
    }

    #[test]
    fn test_update_path_to_root() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);

        let result = tree.move_path("work", "");
        assert!(result.is_ok());

        let paths = tree.get_item_paths(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "tags");
    }

    #[test]
    fn test_add_item_single_path() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);

        let items = tree.get_items("work").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_add_item_multiple_paths() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string(), "urgent".to_string()]);

        let work_items = tree.get_items("work").unwrap();
        assert_eq!(work_items.len(), 1);
        assert!(work_items.contains(&1));

        let urgent_items = tree.get_items("urgent").unwrap();
        assert_eq!(urgent_items.len(), 1);
        assert!(urgent_items.contains(&1));
    }

    #[test]
    fn test_add_item_creates_path() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work/project/urgent".to_string()]);

        let paths = tree.list_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work/project/urgent");

        let items = tree.get_items("work/project/urgent").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_delete_item_single_path() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.delete_item(&1, &["work".to_string()]);

        let items = tree.get_items("work").unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_delete_item_nonexistent_path() {
        let mut tree = Tree::<usize>::new("tags");
        tree.delete_item(&1, &["nonexistent".to_string()]);

        assert_eq!(tree.list_paths().len(), 0);
    }

    #[test]
    fn test_get_items_success() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work".to_string()]);

        let items = tree.get_items("work").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
    }

    #[test]
    fn test_get_items_includes_descendants() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);

        let items = tree.get_items("work").unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
    }

    #[test]
    fn test_get_items_nonexistent_path() {
        let tree = Tree::<usize>::new("tags");
        let result = tree.get_items("nonexistent");
        assert!(result.is_err());
        match result {
            Err(TreeError::PathError(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected PathError"),
        }
    }

    #[test]
    fn test_get_item_paths_single() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);

        let paths = tree.get_item_paths(&1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "work");
    }

    #[test]
    fn test_get_item_paths_multiple() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["b".to_string(), "a".to_string(), "c".to_string()]);

        let paths = tree.get_item_paths(&1);
        assert_eq!(
            paths,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_get_item_paths_nonexistent() {
        let tree = Tree::<usize>::new("tags");
        let paths = tree.get_item_paths(&1);
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_path_normalization_ignores_empty_segments() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["a//b///c".to_string()]);

        let items = tree.get_items("a/b/c").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));

        let paths = tree.list_paths();
        assert_eq!(paths, vec!["a/b/c".to_string()]);
    }

    #[test]
    fn test_move_path_makes_old_path_unreachable() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work//project".to_string()]);

        tree.move_path("work", "job").unwrap();

        let old = tree.get_items("work");
        assert!(matches!(old, Err(TreeError::PathError(_))));

        let items = tree.get_items("job/project").unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.contains(&1));
    }

    #[test]
    fn test_move_path_recreate_old_path_does_not_alias() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work/task".to_string()]);
        tree.add_item(&2, &["job".to_string()]);

        tree.move_path("work", "job").unwrap();

        let old = tree.get_items("work/task");
        assert!(matches!(old, Err(TreeError::PathError(_))));

        let job_task = tree.get_items("job/task").unwrap();
        assert_eq!(job_task.len(), 1);
        assert!(job_task.contains(&1));

        tree.add_item(&3, &["work/task".to_string()]);

        let job_task = tree.get_items("job/task").unwrap();
        assert_eq!(job_task.len(), 1);
        assert!(job_task.contains(&1));

        let work_task = tree.get_items("work/task").unwrap();
        assert_eq!(work_task.len(), 1);
        assert!(work_task.contains(&3));
    }

    #[test]
    fn test_delete_item_nonexistent_item_is_noop() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);

        tree.delete_item(&99, &["work".to_string(), "work/project".to_string()]);

        let work = tree.get_items("work").unwrap();
        assert_eq!(work.len(), 2);
        assert!(work.contains(&1));
        assert!(work.contains(&2));
    }

    #[test]
    fn test_get_bag_only_returns_current_node_items() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);

        let work_bag = tree.get_bag("work").unwrap();
        assert_eq!(work_bag.len(), 1);
        assert!(work_bag.contains(&1));
        assert!(!work_bag.contains(&2));
    }

    #[test]
    fn test_merge_existing_paths_combines_items_and_children() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["job".to_string()]);
        tree.add_item(&3, &["work/project".to_string()]);
        tree.add_item(&4, &["job/project".to_string()]);

        tree.move_path("work", "job").unwrap();

        let job = tree.get_items("job").unwrap();
        assert_eq!(job.len(), 4);
        assert!(job.contains(&1));
        assert!(job.contains(&2));
        assert!(job.contains(&3));
        assert!(job.contains(&4));

        let project = tree.get_items("job/project").unwrap();
        assert_eq!(project.len(), 2);
        assert!(project.contains(&3));
        assert!(project.contains(&4));
    }

    #[test]
    fn test_root_path_listed_when_root_bag_non_empty() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &[String::new()]);

        let paths = tree.list_paths();
        assert_eq!(paths, vec!["tags".to_string()]);
    }

    #[test]
    fn test_get_item_paths_returns_root_name_for_root_items() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &[String::new()]);

        let paths = tree.get_item_paths(&1);
        assert_eq!(paths, vec!["tags".to_string()]);
    }
}
