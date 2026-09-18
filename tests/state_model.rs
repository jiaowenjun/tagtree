use std::collections::{BTreeMap, BTreeSet};

use proptest::{collection::vec, prelude::*, test_runner::TestCaseResult};
use tagtree::{Error, ItemTags, TagNode, TagTree};

const PATHS: [&str; 9] = [
    "alpha",
    "alpha/backend",
    "alpha/backend/rust",
    "alpha/frontend",
    "beta",
    "beta/backend",
    "beta/archive",
    "gamma",
    "gamma/review",
];
const ITEM_COUNT: u8 = 12;

#[derive(Debug, Clone)]
enum Operation {
    SetTags { item: u8, paths: Vec<usize> },
    RemoveItem { item: u8 },
    MoveSubtree { from: usize, to: usize },
    RemoveSubtree { path: usize },
}

fn operations() -> impl Strategy<Value = Vec<Operation>> {
    let operation = prop_oneof![
        5 => (0..ITEM_COUNT, vec(0..PATHS.len(), 0..5))
            .prop_map(|(item, paths)| Operation::SetTags { item, paths }),
        2 => (0..ITEM_COUNT).prop_map(|item| Operation::RemoveItem { item }),
        2 => (0..PATHS.len(), 0..=PATHS.len())
            .prop_map(|(from, to)| Operation::MoveSubtree { from, to }),
        2 => (0..PATHS.len()).prop_map(|path| Operation::RemoveSubtree { path }),
    ];

    vec(operation, 1..80)
}

#[derive(Debug, Default)]
struct ReferenceModel {
    items: BTreeMap<u8, BTreeSet<String>>,
}

impl ReferenceModel {
    fn set_tags(&mut self, item: u8, paths: impl IntoIterator<Item = &'static str>) {
        self.items
            .insert(item, paths.into_iter().map(str::to_string).collect());
    }

    fn remove_item(&mut self, item: u8) {
        self.items.remove(&item);
    }

    fn path_exists(&self, path: &str) -> bool {
        path.is_empty() || self.active_paths().contains(path)
    }

    fn move_subtree(&mut self, from: &str, to: &str) -> Vec<ItemTags<u8>> {
        let affected = self.items_under(from);

        for item in &affected {
            let tags = self.items.get_mut(item).expect("affected item exists");
            *tags = tags
                .iter()
                .map(|tag| move_path(tag, from, to))
                .filter(|tag| !tag.is_empty())
                .collect();
        }

        self.assignments(affected)
    }

    fn remove_subtree(&mut self, path: &str) -> Vec<ItemTags<u8>> {
        let affected = self.items_under(path);

        for item in &affected {
            let tags = self.items.get_mut(item).expect("affected item exists");
            tags.retain(|tag| !is_within(tag, path));
        }

        self.assignments(affected)
    }

    fn items_under(&self, path: &str) -> Vec<u8> {
        let mut items = self
            .items
            .iter()
            .filter_map(|(&item, tags)| {
                (path.is_empty() || tags.iter().any(|tag| is_within(tag, path))).then_some(item)
            })
            .collect::<Vec<_>>();
        items.sort_unstable();
        items
    }

    fn tags_for(&self, item: u8) -> Vec<String> {
        self.items
            .get(&item)
            .map(|tags| tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn assignments(&self, items: Vec<u8>) -> Vec<ItemTags<u8>> {
        items
            .into_iter()
            .map(|item| ItemTags {
                tags: self.tags_for(item),
                item,
            })
            .collect()
    }

    fn active_paths(&self) -> BTreeSet<String> {
        let mut paths = BTreeSet::new();
        for tags in self.items.values() {
            for tag in tags {
                let mut prefix = String::new();
                for segment in tag.split('/') {
                    if !prefix.is_empty() {
                        prefix.push('/');
                    }
                    prefix.push_str(segment);
                    paths.insert(prefix.clone());
                }
            }
        }
        paths
    }

    fn populated_path_count(&self) -> usize {
        self.items
            .values()
            .flat_map(|tags| {
                if tags.is_empty() {
                    vec![String::new()]
                } else {
                    tags.iter().cloned().collect()
                }
            })
            .collect::<BTreeSet<_>>()
            .len()
    }

    fn untagged_item_count(&self) -> usize {
        self.items.values().filter(|tags| tags.is_empty()).count()
    }
}

fn path_at(index: usize) -> &'static str {
    PATHS.get(index).copied().unwrap_or("")
}

fn is_within(candidate: &str, ancestor: &str) -> bool {
    ancestor.is_empty()
        || candidate == ancestor
        || candidate
            .strip_prefix(ancestor)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn is_strict_descendant(candidate: &str, ancestor: &str) -> bool {
    candidate != ancestor && is_within(candidate, ancestor)
}

fn move_path(path: &str, from: &str, to: &str) -> String {
    if !is_within(path, from) {
        return path.to_string();
    }

    let suffix = path.strip_prefix(from).expect("matching prefix");
    if to.is_empty() {
        suffix.strip_prefix('/').unwrap_or(suffix).to_string()
    } else {
        format!("{to}{suffix}")
    }
}

fn snapshot_counts(node: &TagNode, counts: &mut BTreeMap<String, usize>) {
    counts.insert(node.path.clone(), node.item_count);
    for child in &node.children {
        snapshot_counts(child, counts);
    }
}

fn assert_matches_model(tree: &TagTree<u8>, model: &ReferenceModel) -> TestCaseResult {
    prop_assert_eq!(tree.len(), model.items.len());
    prop_assert_eq!(tree.is_empty(), model.items.is_empty());
    for item in 0..ITEM_COUNT {
        prop_assert_eq!(tree.tags_for(&item), model.tags_for(item));
        prop_assert_eq!(tree.contains_item(&item), model.items.contains_key(&item));
    }

    prop_assert_eq!(tree.items_under("").unwrap(), model.items_under(""));
    let active_paths = model.active_paths();
    for path in PATHS {
        if active_paths.contains(path) {
            prop_assert_eq!(tree.items_under(path).unwrap(), model.items_under(path));
        } else {
            prop_assert_eq!(
                tree.items_under(path),
                Err(Error::PathNotFound(path.to_string()))
            );
        }
    }

    let summary = tree.summary();
    prop_assert_eq!(summary.root.item_count, model.items.len());
    prop_assert_eq!(summary.populated_path_count, model.populated_path_count());
    prop_assert_eq!(summary.untagged_item_count, model.untagged_item_count());

    let mut actual_counts = BTreeMap::new();
    snapshot_counts(&summary.root, &mut actual_counts);
    let mut expected_counts = BTreeMap::from([(String::new(), model.items.len())]);
    for path in active_paths {
        expected_counts.insert(path.clone(), model.items_under(&path).len());
    }
    prop_assert_eq!(actual_counts, expected_counts);

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        max_shrink_iters: 4_096,
        ..ProptestConfig::default()
    })]

    #[test]
    fn public_operations_match_the_reference_model(operations in operations()) {
        let mut tree = TagTree::new();
        let mut model = ReferenceModel::default();

        for operation in operations {
            match operation {
                Operation::SetTags { item, paths } => {
                    let paths = paths.into_iter().map(path_at).collect::<Vec<_>>();
                    tree.set_tags(&item, paths.iter().copied()).unwrap();
                    model.set_tags(item, paths);
                }
                Operation::RemoveItem { item } => {
                    tree.remove_item(&item);
                    model.remove_item(item);
                }
                Operation::MoveSubtree { from, to } => {
                    let from = path_at(from);
                    let to = path_at(to);
                    if !model.path_exists(from) {
                        continue;
                    }

                    if is_strict_descendant(to, from) {
                        prop_assert_eq!(
                            tree.move_subtree(from, to),
                            Err(Error::CannotMoveIntoDescendant {
                                from: from.to_string(),
                                to: to.to_string(),
                            })
                        );
                    } else {
                        let expected = model.move_subtree(from, to);
                        prop_assert_eq!(tree.move_subtree(from, to).unwrap(), expected);
                    }
                }
                Operation::RemoveSubtree { path } => {
                    let path = path_at(path);
                    if !model.path_exists(path) {
                        continue;
                    }

                    let expected = model.remove_subtree(path);
                    prop_assert_eq!(tree.remove_subtree(path).unwrap(), expected);
                }
            }

            assert_matches_model(&tree, &model)?;
        }
    }
}
