use tagtree::{is_within, normalize_paths, path_tree::PathTree};

#[test]
fn normalize_paths_trims_deduplicates_and_rejects_invalid_paths() {
    let normalized = normalize_paths([" math ", "math/algebra", "", "math"]).unwrap();
    assert_eq!(normalized, vec!["math", "math/algebra"]);

    assert!(normalize_paths(["/math"]).is_err());
    assert!(normalize_paths(["math/"]).is_err());
    assert!(normalize_paths(["math//algebra"]).is_err());
}

#[test]
fn within_match_is_segment_aware() {
    assert!(is_within("math", "math"));
    assert!(is_within("math/algebra", "math"));
    assert!(!is_within("mathematics", "math"));
}

#[test]
fn path_tree_snapshot_counts_descendants_and_deduplicates_items() {
    let mut tree = PathTree::new("全部标签");
    tree.add_to_paths(&1, &["math".to_string()]);
    tree.add_to_paths(&2, &["math/algebra".to_string(), "math".to_string()]);
    tree.add_to_paths(&3, &[String::new()]);

    let view = tree.snapshot();
    assert_eq!(view.name, "全部标签");
    assert_eq!(view.path, "");
    assert_eq!(view.item_count, 3);
    assert_eq!(tree.root_item_count(), 1);

    let math = view
        .children
        .iter()
        .find(|node| node.path == "math")
        .unwrap();
    assert_eq!(math.item_count, 2);
    assert_eq!(math.children[0].path, "math/algebra");
    assert_eq!(math.children[0].item_count, 1);
}
