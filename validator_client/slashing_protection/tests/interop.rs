use slashing_protection::{interchange::Interchange, SlashingDatabase};
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

fn test_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interchange-tests")
}

#[test]
fn minimal_import_valid() {
    for entry in test_root_dir()
        .join("minimal")
        .join("import")
        .join("valid")
        .read_dir()
        .unwrap()
        .map(Result::unwrap)
    {
        let file = File::open(entry.path()).unwrap();
        let interchange = Interchange::from_json_reader(&file).unwrap();
        let dir = tempdir().unwrap();
        let slashing_db = SlashingDatabase::create(&dir.path().join("slashing_db.sqlite")).unwrap();

        slashing_db
            .import_interchange_info(&interchange, interchange.metadata.genesis_validators_root)
            .unwrap();
    }
}

#[test]
fn minimal_import_invalid() {
    for entry in test_root_dir()
        .join("minimal")
        .join("import")
        .join("invalid")
        .read_dir()
        .unwrap()
        .map(Result::unwrap)
    {
        let file = File::open(entry.path()).unwrap();
        if let Ok(interchange) = Interchange::from_json_reader(&file) {
            let dir = tempdir().unwrap();
            let slashing_db =
                SlashingDatabase::create(&dir.path().join("slashing_db.sqlite")).unwrap();

            slashing_db
                .import_interchange_info(&interchange, interchange.metadata.genesis_validators_root)
                .unwrap_err();
        }
    }
}
