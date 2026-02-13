//! Integration tests for task dependency DAG

use ccboard_core::graph::TaskGraph;
use ccboard_core::models::plan::Task;

fn create_task(id: &str, title: &str, duration: Option<&str>) -> Task {
    Task {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        priority: None,
        duration: duration.map(|s| s.to_string()),
        difficulty: None,
        crate_name: None,
        issue: None,
    }
}

#[test]
fn test_empty_graph() {
    let graph = TaskGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
}

#[test]
fn test_add_single_task() {
    let mut graph = TaskGraph::new();
    graph.add_task(create_task("T1", "Task 1", None));

    assert_eq!(graph.len(), 1);
    assert!(graph.get_task("T1").is_some());
    assert_eq!(graph.get_task("T1").unwrap().title, "Task 1");
}

#[test]
fn test_topological_sort_linear() {
    let mut graph = TaskGraph::new();

    // Linear dependency: T1 -> T2 -> T3 -> T4
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_task(create_task("T3", "Task 3", None));
    graph.add_task(create_task("T4", "Task 4", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T3").unwrap();
    graph.add_dependency("T3", "T4").unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted, vec!["T1", "T2", "T3", "T4"]);
}

#[test]
fn test_topological_sort_diamond() {
    let mut graph = TaskGraph::new();

    // Diamond dependency:
    //     T1
    //    /  \
    //   T2  T3
    //    \  /
    //     T4
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_task(create_task("T3", "Task 3", None));
    graph.add_task(create_task("T4", "Task 4", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T1", "T3").unwrap();
    graph.add_dependency("T2", "T4").unwrap();
    graph.add_dependency("T3", "T4").unwrap();

    let sorted = graph.topological_sort().unwrap();

    // T1 must come first, T4 must come last
    assert_eq!(sorted[0], "T1");
    assert_eq!(sorted[3], "T4");

    // T2 and T3 can be in any order but must come after T1 and before T4
    let t2_pos = sorted.iter().position(|id| id == "T2").unwrap();
    let t3_pos = sorted.iter().position(|id| id == "T3").unwrap();
    assert!(t2_pos > 0 && t2_pos < 3);
    assert!(t3_pos > 0 && t3_pos < 3);
}

#[test]
fn test_cycle_detection_simple() {
    let mut graph = TaskGraph::new();

    // Simple cycle: T1 -> T2 -> T1
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T1").unwrap();

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].len(), 2);

    // Topological sort should fail
    assert!(graph.topological_sort().is_err());
}

#[test]
fn test_cycle_detection_complex() {
    let mut graph = TaskGraph::new();

    // Complex cycle: T1 -> T2 -> T3 -> T1
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_task(create_task("T3", "Task 3", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T3").unwrap();
    graph.add_dependency("T3", "T1").unwrap();

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].len(), 3);
}

#[test]
fn test_no_cycles() {
    let mut graph = TaskGraph::new();

    // DAG with no cycles
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_task(create_task("T3", "Task 3", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T3").unwrap();

    let cycles = graph.detect_cycles();
    assert!(cycles.is_empty());
}

#[test]
fn test_critical_path_linear() {
    let mut graph = TaskGraph::new();

    // Linear with durations: T1(2h) -> T2(3h) -> T3(1h)
    graph.add_task(create_task("T1", "Task 1", Some("2h")));
    graph.add_task(create_task("T2", "Task 2", Some("3h")));
    graph.add_task(create_task("T3", "Task 3", Some("1h")));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T3").unwrap();

    let critical = graph.critical_path().unwrap();
    assert_eq!(critical, vec!["T1", "T2", "T3"]);
}

#[test]
fn test_critical_path_parallel() {
    let mut graph = TaskGraph::new();

    // Parallel paths:
    //     T1(2h)
    //    /      \
    // T2(5h)   T3(2h)
    //    \      /
    //     T4(1h)
    //
    // Critical path: T1 -> T2 -> T4 (total 8h)
    // vs T1 -> T3 -> T4 (total 5h)
    graph.add_task(create_task("T1", "Task 1", Some("2h")));
    graph.add_task(create_task("T2", "Task 2", Some("5h")));
    graph.add_task(create_task("T3", "Task 3", Some("2h")));
    graph.add_task(create_task("T4", "Task 4", Some("1h")));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T1", "T3").unwrap();
    graph.add_dependency("T2", "T4").unwrap();
    graph.add_dependency("T3", "T4").unwrap();

    let critical = graph.critical_path().unwrap();
    // Critical path should be T1 -> T2 -> T4 (longest)
    assert_eq!(critical, vec!["T1", "T2", "T4"]);
}

#[test]
fn test_dependencies_query() {
    let mut graph = TaskGraph::new();

    // T1 -> T2, T1 -> T3
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_task(create_task("T3", "Task 3", None));

    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T1", "T3").unwrap();

    // T2 depends on T1
    let deps = graph.dependencies("T2");
    assert_eq!(deps, vec!["T1"]);

    // T1 has dependents T2 and T3
    let dependents = graph.dependents("T1");
    assert_eq!(dependents.len(), 2);
    assert!(dependents.contains(&"T2".to_string()));
    assert!(dependents.contains(&"T3".to_string()));
}

#[test]
fn test_stress_1000_tasks() {
    let mut graph = TaskGraph::new();

    // Create 1000 tasks in linear chain
    for i in 0..1000 {
        graph.add_task(create_task(
            &format!("T{}", i),
            &format!("Task {}", i),
            None,
        ));
    }

    // Add dependencies: T0 -> T1 -> T2 -> ... -> T999
    for i in 0..999 {
        graph
            .add_dependency(&format!("T{}", i), &format!("T{}", i + 1))
            .unwrap();
    }

    assert_eq!(graph.len(), 1000);

    // Topological sort should work
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 1000);
    assert_eq!(sorted[0], "T0");
    assert_eq!(sorted[999], "T999");

    // No cycles
    assert!(graph.detect_cycles().is_empty());

    // Critical path
    let critical = graph.critical_path().unwrap();
    assert_eq!(critical.len(), 1000);
}

#[test]
fn test_stress_complex_dependencies() {
    let mut graph = TaskGraph::new();

    // Create 100 tasks
    for i in 0..100 {
        graph.add_task(create_task(
            &format!("T{}", i),
            &format!("Task {}", i),
            Some("1h"),
        ));
    }

    // Add complex dependencies (each task depends on previous 3)
    for i in 3..100 {
        graph
            .add_dependency(&format!("T{}", i - 3), &format!("T{}", i))
            .unwrap();
        graph
            .add_dependency(&format!("T{}", i - 2), &format!("T{}", i))
            .unwrap();
        graph
            .add_dependency(&format!("T{}", i - 1), &format!("T{}", i))
            .unwrap();
    }

    assert_eq!(graph.len(), 100);

    // Should still be DAG (no cycles)
    assert!(graph.detect_cycles().is_empty());

    // Topological sort should work
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 100);

    // Critical path should exist
    let critical = graph.critical_path().unwrap();
    assert!(!critical.is_empty());
}

#[test]
fn test_invalid_task_reference() {
    let mut graph = TaskGraph::new();
    graph.add_task(create_task("T1", "Task 1", None));

    // Should fail - T2 doesn't exist
    assert!(graph.add_dependency("T1", "T2").is_err());
}

#[test]
fn test_get_task_not_found() {
    let graph = TaskGraph::new();
    assert!(graph.get_task("nonexistent").is_none());
}

#[test]
fn test_critical_path_with_cycle() {
    let mut graph = TaskGraph::new();

    // Create cycle
    graph.add_task(create_task("T1", "Task 1", None));
    graph.add_task(create_task("T2", "Task 2", None));
    graph.add_dependency("T1", "T2").unwrap();
    graph.add_dependency("T2", "T1").unwrap();

    // Critical path should fail on cyclic graph
    assert!(graph.critical_path().is_err());
}
