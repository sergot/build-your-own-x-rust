mod common;

use crate::common::assert_matches_git;

#[test]
fn test_hash_object() {
    let args = &["hash-object", "-w", "testfile.txt"];
    assert_matches_git(args);
    // TODO: compare files produced?
}
