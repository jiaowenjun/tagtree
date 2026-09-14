use tagtree::tag_index::{TagAssignment, TagIndex};

fn tags(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn upsert_replaces_tags_and_hides_the_untagged_root() {
    let mut index = TagIndex::new();

    index.upsert(&1, &tags(&["work", "life"]));
    index.upsert(&1, &tags(&["work", "life"]));
    assert_eq!(index.items("work").unwrap(), vec![1]);

    index.upsert(&1, &tags(&["study"]));
    assert_eq!(index.tags(&1), tags(&["study"]));
    assert!(index.items("work").unwrap().is_empty());

    index.upsert(&1, &[]);
    assert!(index.tags(&1).is_empty());
    assert_eq!(index.items("").unwrap(), vec![1]);

    index.remove(&1);
    assert!(index.items("").unwrap().is_empty());
}

#[test]
fn items_include_descendants_in_descending_id_order() {
    let mut index = TagIndex::new();
    index.upsert(&1, &tags(&["work"]));
    index.upsert(&3, &tags(&["work/rust"]));
    index.upsert(&2, &tags(&["work/rust/async"]));
    index.upsert(&4, &[]);

    assert_eq!(index.items("work").unwrap(), vec![3, 2, 1]);
    assert_eq!(index.items("").unwrap(), vec![4, 3, 2, 1]);
}

#[test]
fn rename_path_merges_existing_paths_and_returns_persistence_assignments() {
    let mut index = TagIndex::new();
    index.upsert(&1, &tags(&["work/rust"]));
    index.upsert(&2, &tags(&["work/rust/async"]));
    index.upsert(&3, &tags(&["work/backend"]));

    let assignments = index.rename_path("work/rust", "work/backend").unwrap();

    assert_eq!(
        assignments,
        vec![
            TagAssignment {
                item: 2,
                tags: tags(&["work/backend/async"]),
            },
            TagAssignment {
                item: 1,
                tags: tags(&["work/backend"]),
            },
        ],
    );
    assert_eq!(index.items("work/backend").unwrap(), vec![3, 2, 1]);
    assert!(index.items("work/rust").is_err());
}

#[test]
fn delete_path_preserves_other_tags_and_moves_orphans_to_untagged() {
    let mut index = TagIndex::new();
    index.upsert(&1, &tags(&["work/project"]));
    index.upsert(&2, &tags(&["work/project", "life"]));
    index.upsert(&3, &tags(&["workshop"]));

    let assignments = index.delete_path("work").unwrap();

    assert_eq!(
        assignments,
        vec![
            TagAssignment {
                item: 2,
                tags: tags(&["life"]),
            },
            TagAssignment {
                item: 1,
                tags: Vec::new(),
            },
        ],
    );
    assert!(index.tags(&1).is_empty());
    assert_eq!(index.tags(&2), tags(&["life"]));
    assert_eq!(index.tags(&3), tags(&["workshop"]));
    assert_eq!(index.summary().untagged_count, 1);
}

#[test]
fn summary_reports_tree_tag_and_untagged_counts() {
    let mut index = TagIndex::new();
    index.upsert(&1, &tags(&["math"]));
    index.upsert(&2, &tags(&["math", "math/algebra"]));
    index.upsert(&3, &[]);

    let summary = index.summary();

    assert_eq!(summary.tree.item_count, 3);
    assert_eq!(summary.assigned_path_count, 3);
    assert_eq!(summary.untagged_count, 1);
}

#[test]
fn path_mutations_reject_unknown_paths_without_changing_the_index() {
    let mut index = TagIndex::new();
    index.upsert(&1, &tags(&["work"]));

    assert!(index.rename_path("missing", "other").is_err());
    assert!(index.delete_path("missing").is_err());
    assert_eq!(index.tags(&1), tags(&["work"]));
}
