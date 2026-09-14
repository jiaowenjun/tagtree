use std::hash::Hash;

use super::{
    node::NodeId,
    tree::{ROOT_ID, Tree},
};

/// 树视图 (TreeView)
///
/// 用于展示树的层级结构和各节点的数据项统计。
/// 它是一个只读的视图，通常用于 CLI 输出或 UI 展示。
pub struct TreeView {
    /// 节点名称 (例如目录名或标签名)
    pub name: String,
    /// 节点的全路径 (例如 "work/project")
    pub path: String,
    /// 该节点及其子节点包含的数据项总数
    pub item_count: usize,
    /// 子节点视图列表
    pub children: Vec<TreeView>,
}

impl TreeView {
    /// 从 Tree 构建一个 TreeView
    ///
    /// 从根节点开始遍历整棵树，构建层级视图。
    /// 如果根节点没有数据项且没有子节点，会返回一个空的视图。
    pub(crate) fn from_tree<T: Eq + Hash + Clone>(tree: &Tree<T>) -> Self {
        let counts = tree.get_tree_item_counts();
        Self::from_node(tree, ROOT_ID, "", &counts).unwrap_or_else(|| Self {
            name: tree.get_node(ROOT_ID).name().to_string(),
            path: "".to_string(),
            item_count: 0,
            children: Vec::new(),
        })
    }

    /// 将树形结构写入到指定 Writer（仅单测使用）。
    #[cfg(test)]
    fn write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.item_count == 0 {
            writeln!(w, "{} (empty tree)", self.name)?;
        } else {
            writeln!(w, "{} ({})", self.name, self.item_count)?;
        }

        for (i, child) in self.children.iter().enumerate() {
            let is_last = i == self.children.len() - 1;
            child.write_node(w, "", is_last)?;
        }
        Ok(())
    }

    fn from_node<T: Eq + Hash + Clone>(
        tree: &Tree<T>,
        node_id: NodeId,
        parent_path: &str,
        counts: &[usize],
    ) -> Option<Self> {
        let node = tree.get_node(node_id);
        let item_count = counts[node_id];
        if item_count == 0 {
            return None;
        }

        let path = if node_id == ROOT_ID {
            "".to_string()
        } else if parent_path.is_empty() {
            node.name().to_string()
        } else {
            format!("{}/{}", parent_path, node.name())
        };

        let mut children: Vec<TreeView> = node
            .iter_children()
            .filter_map(|&child_id| Self::from_node(tree, child_id, &path, counts))
            .collect();
        children.sort_by(|a, b| a.name.cmp(&b.name));

        Some(Self {
            name: node.name().to_string(),
            item_count,
            children,
            path,
        })
    }

    #[cfg(test)]
    fn write_node<W: std::io::Write>(
        &self,
        w: &mut W,
        prefix: &str,
        is_last: bool,
    ) -> std::io::Result<()> {
        let connector = if is_last { "└── " } else { "├── " };

        if self.item_count == 0 {
            writeln!(w, "{}{}{}", prefix, connector, self.name)?;
        } else {
            writeln!(
                w,
                "{}{}{} ({})",
                prefix, connector, self.name, self.item_count
            )?;
        }

        if !self.children.is_empty() {
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            for (i, child) in self.children.iter().enumerate() {
                let is_last_child = i == self.children.len() - 1;
                child.write_node(w, &new_prefix, is_last_child)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::treebag::Tree;

    #[test]
    fn test_empty_tree_view() {
        let tree = Tree::<usize>::new("tags");
        let view = tree.view();

        assert_eq!(view.name, "tags");
        assert_eq!(view.path, "");
        assert_eq!(view.item_count, 0);
        assert!(view.children.is_empty());
    }

    #[test]
    fn test_tree_view_structure() {
        let mut tree = Tree::<usize>::new("tags");

        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);
        tree.add_item(&3, &["personal".to_string()]);

        let view = tree.view();

        assert_eq!(view.name, "tags");
        assert_eq!(view.item_count, 3);
        assert_eq!(view.children.len(), 2);

        let personal = view.children.iter().find(|c| c.name == "personal").unwrap();
        assert_eq!(personal.path, "personal");
        assert_eq!(personal.item_count, 1);
        assert!(personal.children.is_empty());

        let work = view.children.iter().find(|c| c.name == "work").unwrap();
        assert_eq!(work.path, "work");
        assert_eq!(work.item_count, 2);
        assert_eq!(work.children.len(), 1);

        let project = &work.children[0];
        assert_eq!(project.name, "project");
        assert_eq!(project.path, "work/project");
        assert_eq!(project.item_count, 1);
        assert!(project.children.is_empty());
    }

    #[test]
    fn test_tree_view_deduplication() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&1, &["work/project".to_string()]);

        let view = tree.view();
        let work = view.children.iter().find(|c| c.name == "work").unwrap();

        assert_eq!(work.item_count, 1);

        let project = &work.children[0];
        assert_eq!(project.item_count, 1);
    }

    #[test]
    fn test_tree_view_formatting() {
        let mut tree = Tree::<usize>::new("tags");
        tree.add_item(&1, &["work".to_string()]);
        tree.add_item(&2, &["work/project".to_string()]);
        tree.add_item(&3, &["personal".to_string()]);

        let view = tree.view();
        let mut buffer = Vec::new();
        view.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let expected = "\
tags (3)
├── personal (1)
└── work (2)
    └── project (1)
";
        assert_eq!(output, expected);
    }
}
