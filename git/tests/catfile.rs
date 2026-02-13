mod common;

use crate::common::assert_matches_git;

// use tempdir::TempDir;
#[test]
fn test_catfile_blob() {
    // TODO: setup a git repo and init files
    // let temp_dir = TempDir::new("testgit").unwrap();

    let args = &["cat-file", "-p", "653acc159d4d671f00c148497445085f2f369375"];
    assert_matches_git(args);
}
