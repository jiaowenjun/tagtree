use tagtree::{
    tag_path::{is_descendant_or_self, normalize_tags},
    treebag::Tree,
};

#[test]
fn normalize_tags_trims_deduplicates_and_rejects_invalid_paths() {
    let normalized = normalize_tags([" math ", "math/algebra", "", "math"]).unwrap();
    assert_eq!(normalized, vec!["math", "math/algebra"]);

    assert!(normalize_tags(["/math"]).is_err());
    assert!(normalize_tags(["math/"]).is_err());
    assert!(normalize_tags(["math//algebra"]).is_err());
}

#[test]
fn descendant_match_is_segment_aware() {
    assert!(is_descendant_or_self("math", "math"));
    assert!(is_descendant_or_self("math/algebra", "math"));
    assert!(!is_descendant_or_self("mathematics", "math"));
}

#[test]
fn treebag_view_counts_descendants_and_deduplicates_items() {
    let mut tree = Tree::new("全部标签");
    tree.add_item(&1, &["math".to_string()]);
    tree.add_item(&2, &["math/algebra".to_string(), "math".to_string()]);
    tree.add_item(&3, &[String::new()]);

    let view = tree.view();
    assert_eq!(view.name, "全部标签");
    assert_eq!(view.path, "");
    assert_eq!(view.item_count, 3);
    assert_eq!(tree.root_bag_count(), 1);

    let math = view
        .children
        .iter()
        .find(|node| node.path == "math")
        .unwrap();
    assert_eq!(math.item_count, 2);
    assert_eq!(math.children[0].path, "math/algebra");
    assert_eq!(math.children[0].item_count, 1);
}
