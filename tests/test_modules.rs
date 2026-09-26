use cuv::compiler::modules::ModuleDAG;
use std::fs;
use std::path::PathBuf;

fn create_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cuv_test_modules_{}_{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_cxx20_module_scanning_and_dag() {
    let temp_dir = create_temp_dir("dag_scan");

    let mod_math = temp_dir.join("math.cppm");
    fs::write(
        &mod_math,
        r#"
export module math;
export int add(int a, int b) { return a + b; }
"#,
    )
    .unwrap();

    let mod_calc = temp_dir.join("calc.cppm");
    fs::write(
        &mod_calc,
        r#"
export module calc;
import math;
export int compute(int x) { return add(x, 10); }
"#,
    )
    .unwrap();

    let main_cpp = temp_dir.join("main.cpp");
    fs::write(
        &main_cpp,
        r#"
#include <iostream>
import calc;
int main() { return compute(5); }
"#,
    )
    .unwrap();

    let sources = vec![mod_math.clone(), mod_calc.clone(), main_cpp.clone()];
    let dag = ModuleDAG::scan(&sources).expect("scan sources");

    assert!(dag.has_modules());
    assert_eq!(dag.modules.len(), 2);
    assert!(dag.modules.contains_key("math"));
    assert!(dag.modules.contains_key("calc"));

    let stages = dag.topological_stages().expect("topological sort");
    assert_eq!(stages.len(), 2, "Expected 2 dependency stages");

    // Stage 0 must contain 'math' (0 dependencies)
    assert_eq!(stages[0].len(), 1);
    assert_eq!(stages[0][0].module_name.as_deref(), Some("math"));

    // Stage 1 must contain 'calc' (depends on 'math')
    assert_eq!(stages[1].len(), 1);
    assert_eq!(stages[1][0].module_name.as_deref(), Some("calc"));

    // Consumers check
    assert_eq!(dag.consumers.len(), 1);
    assert_eq!(dag.consumers[0].imported_modules, vec!["calc"]);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cxx20_module_cycle_detection() {
    let temp_dir = create_temp_dir("cycle");

    let mod_a = temp_dir.join("a.cppm");
    fs::write(
        &mod_a,
        r#"
export module a;
import b;
"#,
    )
    .unwrap();

    let mod_b = temp_dir.join("b.cppm");
    fs::write(
        &mod_b,
        r#"
export module b;
import a;
"#,
    )
    .unwrap();

    let sources = vec![mod_a, mod_b];
    let dag = ModuleDAG::scan(&sources).expect("scan sources");
    let result = dag.topological_stages();

    assert!(result.is_err(), "Expected cyclic module dependency error");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cyclic dependency"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_flag_synthesis() {
    let modules_dir = PathBuf::from("/path/to/target/debug/modules");
    let src = PathBuf::from("src/math.cppm");
    let pcm = modules_dir.join("math.pcm");

    let precompile_args = ModuleDAG::precompile_args("c++20", &modules_dir, &src, &pcm);
    assert!(precompile_args.contains(&"--precompile".to_string()));
    assert!(
        precompile_args.contains(&"-xc++-module".to_string())
            || precompile_args.contains(&"c++-module".to_string())
    );
    assert!(precompile_args
        .contains(&"-fprebuilt-module-path=/path/to/target/debug/modules".to_string()));

    let consumer_args = ModuleDAG::consumer_module_args(&modules_dir);
    assert_eq!(
        consumer_args,
        vec!["-fprebuilt-module-path=/path/to/target/debug/modules"]
    );
}
