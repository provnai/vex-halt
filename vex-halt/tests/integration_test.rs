//! Integration tests for VEX-HALT benchmark

use std::path::PathBuf;

/// Get the path to the test dataset
fn get_dataset_path() -> PathBuf {
    // When running tests, we're in the project root
    PathBuf::from("datasets/vex_halt")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dataset_directory_exists() {
        let path = get_dataset_path();
        assert!(path.exists(), "Dataset directory should exist at {:?}", path);
    }
    
    #[test]
    fn test_index_json_exists() {
        let path = get_dataset_path().join("index.json");
        assert!(path.exists(), "index.json should exist at {:?}", path);
    }
    
    #[test]
    fn test_all_category_directories_exist() {
        let base = get_dataset_path();
        let categories = ["cct", "api", "fct", "hht", "rt", "frontier", "vsm", "mtc", "eas", "mem", "agt", "vex"];
        
        for cat in &categories {
            let cat_path = base.join(cat);
            assert!(cat_path.exists(), "Category directory {:?} should exist", cat_path);
        }
    }
    
    #[test]
    fn test_cct_files_exist() {
        let base = get_dataset_path().join("cct");
        let files = ["easy.json", "medium.json", "hard.json", "ambiguous.json", "unanswerable.json"];
        
        for file in &files {
            let path = base.join(file);
            assert!(path.exists(), "CCT file {:?} should exist", path);
        }
    }
    
    #[test]
    fn test_json_files_are_valid() {
        let base = get_dataset_path();
        
        // Test a few key files are valid JSON
        let test_files = [
            "index.json",
            "cct/easy.json",
            "api/direct_injection.json",
            "frontier/adversarial_reasoning.json",
        ];
        
        for file in &test_files {
            let path = base.join(file);
            if path.exists() {
                let content = std::fs::read_to_string(&path)
                    .expect(&format!("Should be able to read {:?}", path));
                let _: serde_json::Value = serde_json::from_str(&content)
                    .expect(&format!("{:?} should be valid JSON", path));
            }
        }
    }
    
    #[test]
    fn test_index_has_required_fields() {
        let path = get_dataset_path().join("index.json");
        let content = std::fs::read_to_string(&path).expect("Should read index.json");
        let index: serde_json::Value = serde_json::from_str(&content).expect("Should parse index.json");
        
        assert!(index.get("name").is_some(), "index.json should have 'name' field");
        assert!(index.get("version").is_some(), "index.json should have 'version' field");
        assert!(index.get("statistics").is_some(), "index.json should have 'statistics' field");
    }
}


