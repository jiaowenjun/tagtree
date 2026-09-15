use std::{collections::HashSet, hash::Hash};

pub(crate) type NodeId = usize;

/// 树节点：每个节点维护一个背包（HashSet），用于存放该节点关联的数据项
#[derive(Debug)]
pub(crate) struct Node<T> {
    /// 节点名
    name: String,
    /// 父节点索引，0 表示根节点
    parent_id: NodeId,
    /// 子节点索引集合
    child_ids: HashSet<NodeId>,
    /// 节点背包（存放该节点关联的数据项）
    bag: HashSet<T>,
}

impl<T: Eq + Hash> Node<T> {
    /// 【构造函数】创建新节点，指定父节点索引
    pub fn new(name: &str, parent_id: NodeId) -> Self {
        Self {
            name: name.to_string(),
            bag: HashSet::new(),
            parent_id,
            child_ids: HashSet::new(),
        }
    }

    /// 【构造函数】创建根节点，根结点父节点是它自己
    pub fn new_root(name: &str) -> Self {
        Self::new(name, 0)
    }

    /// 【节点属性访问器】节点名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 【节点属性访问器】父节点索引
    pub fn parent_id(&self) -> NodeId {
        self.parent_id
    }

    /// 【节点属性修改器】设置父节点索引
    pub fn set_parent_id(&mut self, parent_id: NodeId) {
        self.parent_id = parent_id;
    }

    /// 【子节点管理】遍历子节点
    pub fn iter_children(&self) -> impl Iterator<Item = &NodeId> {
        self.child_ids.iter()
    }

    /// 【子节点管理】关联子节点
    pub fn link_child(&mut self, child_id: NodeId) {
        self.child_ids.insert(child_id);
    }

    /// 【子节点管理】删除子节点关联
    pub fn unlink_child(&mut self, child_id: &NodeId) {
        self.child_ids.remove(child_id);
    }

    /// 【背包操作】判断背包是否为空
    pub fn is_empty(&self) -> bool {
        self.bag.is_empty()
    }

    /// 【背包操作】取出节点背包（转移所有权）
    pub fn take_bag(&mut self) -> HashSet<T> {
        std::mem::take(&mut self.bag)
    }

    /// 【背包操作】合并另一个节点的背包
    pub fn merge_bag(&mut self, other: HashSet<T>) {
        self.bag.extend(other);
    }

    /// 【背包操作】添加数据项到背包
    pub fn add_item(&mut self, item: T) {
        self.bag.insert(item);
    }

    /// 【背包操作】从背包删除数据项
    pub fn remove_item(&mut self, item: &T) {
        self.bag.remove(item);
    }

    /// 【背包操作】遍历背包中的数据项（不含子节点）
    pub fn iter_bag(&self) -> impl Iterator<Item = &T> {
        self.bag.iter()
    }

    /// 【背包操作】是否包含某个数据项
    pub fn contains(&self, item: &T) -> bool {
        self.bag.contains(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_node() {
        let node = Node::<usize>::new("work", 0);
        assert_eq!(node.name(), "work");
        assert_eq!(node.parent_id(), 0);
        assert!(node.is_empty());
    }

    #[test]
    fn test_new_root() {
        let root = Node::<usize>::new_root("root");
        assert_eq!(root.name(), "root");
        assert_eq!(root.parent_id(), 0);
        assert!(root.is_empty());
    }

    #[test]
    fn test_set_parent_id() {
        let mut node = Node::<usize>::new("urgent", 1);
        assert_eq!(node.parent_id(), 1);
        node.set_parent_id(2);
        assert_eq!(node.parent_id(), 2);
    }

    #[test]
    fn test_child_management() {
        let mut parent = Node::<usize>::new("work", 0);
        assert_eq!(parent.iter_children().count(), 0);

        parent.link_child(1);
        parent.link_child(2);
        parent.link_child(3);

        let children: Vec<_> = parent.iter_children().copied().collect();
        assert_eq!(children.len(), 3);
        assert!(children.contains(&1));
        assert!(children.contains(&2));
        assert!(children.contains(&3));

        parent.unlink_child(&1);
        assert_eq!(parent.iter_children().count(), 2);
        let children: Vec<_> = parent.iter_children().copied().collect();
        assert!(!children.contains(&1));
        assert!(children.contains(&2));
        assert!(children.contains(&3));

        parent.unlink_child(&99);
        assert_eq!(parent.iter_children().count(), 2);
    }

    #[test]
    fn test_bag_operations() {
        let mut node = Node::<usize>::new("work", 0);
        assert!(node.is_empty());
        assert!(!node.contains(&1));

        node.add_item(1);
        assert!(!node.is_empty());
        assert!(node.contains(&1));

        node.add_item(2);
        assert!(node.contains(&1));
        assert!(node.contains(&2));

        node.add_item(2);
        assert_eq!(node.iter_bag().count(), 2);
        assert!(node.contains(&1));
        assert!(node.contains(&2));

        let items: Vec<_> = node.iter_bag().copied().collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&1));
        assert!(items.contains(&2));

        node.remove_item(&1);
        assert!(!node.contains(&1));
        assert!(node.contains(&2));

        node.remove_item(&99);
        assert_eq!(node.iter_bag().count(), 1);
        assert!(node.contains(&2));
    }

    #[test]
    fn test_take_bag() {
        let mut node = Node::<usize>::new("work", 0);
        node.add_item(1);
        node.add_item(2);
        node.add_item(3);

        assert!(!node.is_empty());
        let bag = node.take_bag();
        assert!(node.is_empty());
        assert_eq!(bag.len(), 3);
        assert!(bag.contains(&1));
        assert!(bag.contains(&2));
        assert!(bag.contains(&3));
    }

    #[test]
    fn test_merge_bag() {
        let mut node1 = Node::<usize>::new("work", 0);
        node1.add_item(1);
        node1.add_item(2);

        let mut node2 = Node::<usize>::new("project", 1);
        node2.add_item(2);
        node2.add_item(3);
        node2.add_item(4);

        let bag2 = node2.take_bag();
        node1.merge_bag(bag2);

        assert_eq!(node1.iter_children().count(), 0);
        assert!(node1.contains(&1));
        assert!(node1.contains(&2));
        assert!(node1.contains(&3));
        assert!(node1.contains(&4));
    }

    #[test]
    fn test_duplicate_items_in_bag() {
        let mut node = Node::<usize>::new("work", 0);
        node.add_item(1);
        node.add_item(1);
        node.add_item(2);

        assert_eq!(node.iter_bag().count(), 2);
        assert!(node.contains(&1));
        assert!(node.contains(&2));
    }

    #[test]
    fn test_empty_bag_operations() {
        let mut node = Node::<usize>::new("work", 0);

        assert!(node.is_empty());
        assert_eq!(node.iter_bag().count(), 0);

        let bag = node.take_bag();
        assert!(bag.is_empty());

        node.merge_bag(std::collections::HashSet::new());
        assert!(node.is_empty());
    }

    #[test]
    fn test_multiple_children_management() {
        let mut parent = Node::<usize>::new("work", 0);

        for i in 1..=10 {
            parent.link_child(i);
        }

        assert_eq!(parent.iter_children().count(), 10);

        parent.unlink_child(&5);
        assert_eq!(parent.iter_children().count(), 9);

        let children: Vec<_> = parent.iter_children().copied().collect();
        assert!(!children.contains(&5));
        assert!(children.contains(&1));
        assert!(children.contains(&10));
    }

    #[test]
    fn test_comprehensive_tag_tree_scenario() {
        let root = Node::<usize>::new_root("root");
        assert_eq!(root.name(), "root");
        assert_eq!(root.parent_id(), 0);

        let mut work = Node::<usize>::new("work", 0);
        work.add_item(1);
        work.add_item(2);
        work.add_item(3);

        let mut project = Node::<usize>::new("project", 1);
        project.add_item(2);
        project.add_item(4);

        let mut urgent = Node::<usize>::new("urgent", 2);
        urgent.add_item(4);
        urgent.add_item(5);

        assert_eq!(work.name(), "work");
        assert_eq!(work.parent_id(), 0);
        assert_eq!(work.iter_children().count(), 0);
        assert_eq!(work.iter_bag().count(), 3);

        work.link_child(1);
        assert_eq!(work.iter_children().count(), 1);

        assert_eq!(project.name(), "project");
        assert_eq!(project.parent_id(), 1);
        assert_eq!(project.iter_bag().count(), 2);

        project.link_child(2);
        assert_eq!(project.iter_children().count(), 1);

        assert_eq!(urgent.name(), "urgent");
        assert_eq!(urgent.parent_id(), 2);
        assert_eq!(urgent.iter_bag().count(), 2);

        let mut new_work = Node::<usize>::new("work", 0);
        new_work.add_item(6);
        new_work.add_item(7);

        let work_bag = work.take_bag();
        new_work.merge_bag(work_bag);
        assert_eq!(new_work.iter_bag().count(), 5);
        assert!(new_work.contains(&1));
        assert!(new_work.contains(&2));
        assert!(new_work.contains(&3));
        assert!(new_work.contains(&6));
        assert!(new_work.contains(&7));
    }
}
