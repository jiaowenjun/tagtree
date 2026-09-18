use std::error::Error as _;

use tagtree::{Error, ItemTags, TagTree};

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
    assert!(matches!(
        index.items_under("work"),
        Err(Error::PathNotFound(path)) if path == "work"
    ));

    index.set_tags(&1, std::iter::empty::<&str>()).unwrap();
    assert!(index.tags_for(&1).is_empty());
    assert_eq!(index.items_under("").unwrap(), vec![1]);

    index.remove_item(&1);
    assert!(index.items_under("").unwrap().is_empty());
}

#[test]
fn item_inspection_tracks_tagged_and_untagged_items() {
    let mut index = TagTree::new();
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
    assert!(!index.contains_item(&1));

    index.set_tags(&1, ["work"]).unwrap();
    index.set_tags(&2, std::iter::empty::<&str>()).unwrap();

    assert!(!index.is_empty());
    assert_eq!(index.len(), 2);
    assert!(index.contains_item(&1));
    assert!(index.contains_item(&2));

    index.remove_item(&1);
    assert_eq!(index.len(), 1);
    assert!(!index.contains_item(&1));

    index.remove_item(&2);
    assert!(index.is_empty());
}

#[test]
fn removing_the_last_item_prunes_its_empty_path() {
    let mut index = TagTree::new();
    index.set_tags(&1, ["work/rust"]).unwrap();

    index.remove_item(&1);

    for path in ["work", "work/rust"] {
        assert!(matches!(
            index.items_under(path),
            Err(Error::PathNotFound(missing)) if missing == path
        ));
    }
}

#[test]
fn items_include_descendants_in_ascending_order() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work"])).unwrap();
    index.set_tags(&3, tags(&["work/rust"])).unwrap();
    index.set_tags(&2, tags(&["work/rust/async"])).unwrap();
    index.set_tags(&4, std::iter::empty::<&str>()).unwrap();

    assert_eq!(index.items_under("work").unwrap(), vec![1, 2, 3]);
    assert_eq!(index.items_under("").unwrap(), vec![1, 2, 3, 4]);
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
                item: 1,
                tags: tags(&["work/backend"]),
            },
            ItemTags {
                item: 2,
                tags: tags(&["work/backend/async"]),
            },
        ],
    );
    assert_eq!(index.items_under("work/backend").unwrap(), vec![1, 2, 3]);
    assert!(index.items_under("work/rust").is_err());
}

#[test]
fn moving_a_subtree_to_root_only_untags_items_without_other_tags() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work", "life"])).unwrap();
    index.set_tags(&2, tags(&["work"])).unwrap();

    let assignments = index.move_subtree("work", "").unwrap();

    assert_eq!(
        assignments,
        vec![
            ItemTags {
                item: 1,
                tags: tags(&["life"]),
            },
            ItemTags {
                item: 2,
                tags: Vec::new(),
            },
        ],
    );
    assert_eq!(index.summary().untagged_item_count, 1);
    assert_eq!(index.summary().populated_path_count, 2);
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
                item: 1,
                tags: Vec::new(),
            },
            ItemTags {
                item: 2,
                tags: tags(&["life"]),
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

#[test]
fn invalid_path_errors_remain_available_in_the_error_chain() {
    let error = TagTree::<u8>::new().items_under("/invalid").unwrap_err();

    let source = error.source().expect("invalid path should be the source");
    assert_eq!(source.to_string(), "invalid tag path: /invalid");
}

#[test]
fn move_subtree_rejection_leaves_the_tree_unchanged() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work"])).unwrap();
    index.set_tags(&2, tags(&["work/rust"])).unwrap();

    assert!(matches!(
        index.move_subtree("work", "work/rust/deep").unwrap_err(),
        Error::CannotMoveIntoDescendant { .. }
    ));
    for path in ["work/rust/deep", "work/rust/deep/x"] {
        assert!(matches!(
            index.items_under(path),
            Err(Error::PathNotFound(_))
        ));
    }

    assert!(matches!(
        index.move_subtree("", "other").unwrap_err(),
        Error::CannotMoveRoot
    ));
    assert!(matches!(
        index.items_under("other"),
        Err(Error::PathNotFound(_))
    ));

    assert_eq!(index.items_under("work").unwrap(), vec![1, 2]);
    assert_eq!(index.tags_for(&1), tags(&["work"]));
    assert_eq!(index.tags_for(&2), tags(&["work/rust"]));
}

#[test]
fn remove_subtree_rejects_root_without_changing_items() {
    let mut index = TagTree::new();
    index.set_tags(&1, tags(&["work"])).unwrap();
    index.set_tags(&2, std::iter::empty::<&str>()).unwrap();

    assert!(matches!(
        index.remove_subtree("").unwrap_err(),
        Error::CannotRemoveRoot
    ));
    assert_eq!(index.tags_for(&1), tags(&["work"]));
    assert_eq!(index.items_under("").unwrap(), vec![1, 2]);
}
