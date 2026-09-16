use tagtree::{is_within, normalize_paths};

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
