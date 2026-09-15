use tagtree::{ItemTags, TagTree};

fn tags(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn set_tags_replaces_tags_and_hides_the_untagged_root() {
    let mut index = TagTree::new();

    index.set_tags(&1, tags(&["work", "life"])).unwrap();
    index.set_tags(&1, tags(&["work", "life"])).unwrap();
    assert_eq!(index.items_under("work").unwrap(), vec![1]);

    index.set_tags(&1, tags(&["study"])).unwrap();
    assert_eq!(index.tags_for(&1), tags(&["study"]));
    assert!(index.items_under("work").unwrap().is_empty());

    index.set_tags(&1, std::iter::empty::<&str>()).unwrap();
    assert!(index.tags_for(&1).is_empty());
    assert_eq!(index.items_under("").unwrap(), vec![1]);

    index.remove_item(&1);
    assert!(index.items_under("").unwrap().is_empty());
}

#[test]
fn items_include_descendants_in_descending_id_order() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work"])).unwrap();
    index.set_tags(&3, tags(&["work/rust"])).unwrap();
    index.set_tags(&2, tags(&["work/rust/async"])).unwrap();
    index.set_tags(&4, std::iter::empty::<&str>()).unwrap();

    assert_eq!(index.items_under("work").unwrap(), vec![3, 2, 1]);
    assert_eq!(index.items_under("").unwrap(), vec![4, 3, 2, 1]);
}

#[test]
fn move_subtree_merges_existing_paths_and_returns_item_tags() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work/rust"])).unwrap();
    index.set_tags(&2, tags(&["work/rust/async"])).unwrap();
    index.set_tags(&3, tags(&["work/backend"])).unwrap();

    let assignments = index.move_subtree("work/rust", "work/backend").unwrap();

    assert_eq!(
        assignments,
        vec![
            ItemTags {
                item: 2,
                tags: tags(&["work/backend/async"]),
            },
            ItemTags {
                item: 1,
                tags: tags(&["work/backend"]),
            },
        ],
    );
    assert_eq!(index.items_under("work/backend").unwrap(), vec![3, 2, 1]);
    assert!(index.items_under("work/rust").is_err());
}

#[test]
fn remove_subtree_preserves_other_tags_and_moves_orphans_to_untagged() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work/project"])).unwrap();
    index.set_tags(&2, tags(&["work/project", "life"])).unwrap();
    index.set_tags(&3, tags(&["workshop"])).unwrap();

    let assignments = index.remove_subtree("work").unwrap();

    assert_eq!(
        assignments,
        vec![
            ItemTags {
                item: 2,
                tags: tags(&["life"]),
            },
            ItemTags {
                item: 1,
                tags: Vec::new(),
            },
        ],
    );
    assert!(index.tags_for(&1).is_empty());
    assert_eq!(index.tags_for(&2), tags(&["life"]));
    assert_eq!(index.tags_for(&3), tags(&["workshop"]));
    assert_eq!(index.summary().untagged_item_count, 1);
}

#[test]
fn summary_reports_tree_tag_and_untagged_counts() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["math"])).unwrap();
    index.set_tags(&2, tags(&["math", "math/algebra"])).unwrap();
    index.set_tags(&3, std::iter::empty::<&str>()).unwrap();

    let summary = index.summary();

    assert_eq!(summary.root.item_count, 3);
    assert_eq!(summary.populated_path_count, 3);
    assert_eq!(summary.untagged_item_count, 1);
}

#[test]
fn path_mutations_reject_unknown_paths_without_changing_the_index() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work"])).unwrap();

    assert!(index.move_subtree("missing", "other").is_err());
    assert!(index.remove_subtree("missing").is_err());
    assert_eq!(index.tags_for(&1), tags(&["work"]));
}

#[test]
fn set_tags_rejects_invalid_paths_without_changing_the_item() {
    let mut tree = TagTree::new();
    tree.set_tags(&1, ["work/rust"]).unwrap();

    assert!(tree.set_tags(&1, ["/invalid"]).is_err());
    assert_eq!(tree.tags_for(&1), tags(&["work/rust"]));
}
