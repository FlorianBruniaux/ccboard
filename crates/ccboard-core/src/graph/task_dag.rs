//! Task dependency graph using petgraph
//!
//! Provides DAG (Directed Acyclic Graph) operations for task dependencies:
//! - Topological sort for execution order
//! - Cycle detection for circular dependencies
//! - Critical path analysis for bottleneck identification
//!
//! # Example
//!
//! ```
//! use ccboard_core::graph::TaskGraph;
//! use ccboard_core::models::plan::Task;
//!
//! // Create graph
//! let mut graph = TaskGraph::new();
//!
//! // Add tasks
//! let task1 = Task {
//!     id: "T1".to_string(),
//!     title: "Design".to_string(),
//!     duration: Some("2h".to_string()),
//!     ..Default::default()
//! };
//! let task2 = Task {
//!     id: "T2".to_string(),
//!     title: "Implementation".to_string(),
//!     duration: Some("5h".to_string()),
//!     ..Default::default()
//! };
//!
//! graph.add_task(task1);
//! graph.add_task(task2);
//!
//! // Add dependency: T1 must complete before T2
//! graph.add_dependency("T1", "T2").unwrap();
//!
//! // Get execution order
//! let order = graph.topological_sort().unwrap();
//! assert_eq!(order, vec!["T1", "T2"]);
//!
//! // Check for cycles
//! assert!(graph.detect_cycles().is_empty());
//!
//! // Find critical path
//! let critical = graph.critical_path().unwrap();
//! assert_eq!(critical, vec!["T1", "T2"]); // Both on critical path
//! ```

use crate::models::plan::Task;
use anyhow::{Context, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

/// Edge type representing task dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyEdge {
    /// Task A blocks Task B (B depends on A)
    Blocks,
    /// Task A is blocked by Task B (A depends on B)
    BlockedBy,
}

/// Task dependency graph
///
/// Wraps petgraph DiGraph to provide task-specific operations.
/// Nodes are Tasks, edges are DependencyEdge relationships.
pub struct TaskGraph {
    /// Internal directed graph
    graph: DiGraph<Task, DependencyEdge>,

    /// Map from task ID to node index
    task_index: HashMap<String, NodeIndex>,
}

impl TaskGraph {
    /// Create a new empty task graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            task_index: HashMap::new(),
        }
    }

    /// Add a task to the graph
    ///
    /// Returns the node index for the added task.
    pub fn add_task(&mut self, task: Task) -> NodeIndex {
        let task_id = task.id.clone();
        let node = self.graph.add_node(task);
        self.task_index.insert(task_id, node);
        node
    }

    /// Add a dependency: task_a blocks task_b (b depends on a)
    ///
    /// Returns error if either task ID not found.
    pub fn add_dependency(&mut self, task_a_id: &str, task_b_id: &str) -> Result<()> {
        let node_a = self
            .task_index
            .get(task_a_id)
            .copied()
            .context(format!("Task not found: {}", task_a_id))?;

        let node_b = self
            .task_index
            .get(task_b_id)
            .copied()
            .context(format!("Task not found: {}", task_b_id))?;

        self.graph.add_edge(node_a, node_b, DependencyEdge::Blocks);
        Ok(())
    }

    /// Perform topological sort
    ///
    /// Returns tasks in valid execution order (dependencies first).
    /// Returns error if graph contains cycles.
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        use petgraph::algo::toposort;

        let sorted_nodes = toposort(&self.graph, None)
            .map_err(|cycle| anyhow::anyhow!("Cycle detected at node {:?}", cycle.node_id()))?;

        let task_ids = sorted_nodes
            .into_iter()
            .map(|node| self.graph[node].id.clone())
            .collect();

        Ok(task_ids)
    }

    /// Detect cycles in the graph
    ///
    /// Returns list of cycles, where each cycle is a list of task IDs.
    /// Empty list means no cycles (valid DAG).
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        use petgraph::algo::kosaraju_scc;

        let sccs = kosaraju_scc(&self.graph);

        // Filter SCCs with more than one node (cycles)
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                scc.into_iter()
                    .map(|node| self.graph[node].id.clone())
                    .collect()
            })
            .collect()
    }

    /// Calculate critical path (longest path in DAG)
    ///
    /// Returns list of task IDs on the critical path.
    /// Critical path represents the bottleneck (minimum time to complete all tasks).
    ///
    /// Uses task duration estimates if available, otherwise treats all tasks as duration=1.
    pub fn critical_path(&self) -> Result<Vec<String>> {
        // First verify no cycles
        if !self.detect_cycles().is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot compute critical path: graph contains cycles"
            ));
        }

        // Get topological order
        let topo_order = self.topological_sort()?;

        // Map task ID to node index
        let id_to_node: HashMap<_, _> = self
            .task_index
            .iter()
            .map(|(id, &node)| (id.clone(), node))
            .collect();

        // Compute longest path using dynamic programming
        let mut distances: HashMap<NodeIndex, f64> = HashMap::new();
        let mut predecessors: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        // Initialize all distances to 0
        for node in self.graph.node_indices() {
            distances.insert(node, 0.0);
        }

        // Process nodes in topological order
        for task_id in topo_order {
            let node = id_to_node[&task_id];
            let task = &self.graph[node];

            // Parse duration from task (e.g., "3-4h" → 3.5)
            let duration = Self::parse_duration(&task.duration);

            let current_distance = distances[&node];

            // Update distances for all successors
            for edge in self.graph.edges_directed(node, Direction::Outgoing) {
                let successor = edge.target();
                let new_distance = current_distance + duration;

                if new_distance > distances[&successor] {
                    distances.insert(successor, new_distance);
                    predecessors.insert(successor, node);
                }
            }
        }

        // Find node with maximum distance (end of critical path)
        let (&end_node, _) = distances
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .context("No tasks in graph")?;

        // Backtrack to reconstruct critical path
        let mut path = vec![end_node];
        let mut current = end_node;

        while let Some(&pred) = predecessors.get(&current) {
            path.push(pred);
            current = pred;
        }

        path.reverse();

        // Convert node indices to task IDs
        let critical_path_ids = path
            .into_iter()
            .map(|node| self.graph[node].id.clone())
            .collect();

        Ok(critical_path_ids)
    }

    /// Parse duration string to hours (e.g., "3-4h" → 3.5)
    fn parse_duration(duration: &Option<String>) -> f64 {
        duration
            .as_ref()
            .and_then(|s| {
                // Extract first number from string like "3-4h" or "2h"
                let num_str: String = s.chars().take_while(|c| c.is_numeric()).collect();
                num_str.parse::<f64>().ok()
            })
            .unwrap_or(1.0) // Default to 1 hour if no duration
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.task_index.get(task_id).map(|&node| &self.graph[node])
    }

    /// Get all tasks
    pub fn tasks(&self) -> Vec<&Task> {
        self.graph.node_weights().collect()
    }

    /// Get dependencies for a task (tasks that this task depends on)
    pub fn dependencies(&self, task_id: &str) -> Vec<String> {
        self.task_index
            .get(task_id)
            .map(|&node| {
                self.graph
                    .edges_directed(node, Direction::Incoming)
                    .map(|edge| self.graph[edge.source()].id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get dependents for a task (tasks that depend on this task)
    pub fn dependents(&self, task_id: &str) -> Vec<String> {
        self.task_index
            .get(task_id)
            .map(|&node| {
                self.graph
                    .edges_directed(node, Direction::Outgoing)
                    .map(|edge| self.graph[edge.target()].id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Number of tasks in graph
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(id: &str, title: &str, duration: Option<&str>) -> Task {
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
    fn test_add_task() {
        let mut graph = TaskGraph::new();
        let task = create_test_task("T1", "Task 1", None);
        graph.add_task(task);

        assert_eq!(graph.len(), 1);
        assert!(graph.get_task("T1").is_some());
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut graph = TaskGraph::new();

        // T1 -> T2 -> T3
        graph.add_task(create_test_task("T1", "Task 1", None));
        graph.add_task(create_test_task("T2", "Task 2", None));
        graph.add_task(create_test_task("T3", "Task 3", None));

        graph.add_dependency("T1", "T2").unwrap();
        graph.add_dependency("T2", "T3").unwrap();

        let sorted = graph.topological_sort().unwrap();

        // T1 must come before T2, T2 before T3
        let t1_pos = sorted.iter().position(|id| id == "T1").unwrap();
        let t2_pos = sorted.iter().position(|id| id == "T2").unwrap();
        let t3_pos = sorted.iter().position(|id| id == "T3").unwrap();

        assert!(t1_pos < t2_pos);
        assert!(t2_pos < t3_pos);
    }

    #[test]
    fn test_detect_cycles_none() {
        let mut graph = TaskGraph::new();

        graph.add_task(create_test_task("T1", "Task 1", None));
        graph.add_task(create_test_task("T2", "Task 2", None));
        graph.add_dependency("T1", "T2").unwrap();

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_detect_cycles_simple() {
        let mut graph = TaskGraph::new();

        // Create cycle: T1 -> T2 -> T1
        graph.add_task(create_test_task("T1", "Task 1", None));
        graph.add_task(create_test_task("T2", "Task 2", None));
        graph.add_dependency("T1", "T2").unwrap();
        graph.add_dependency("T2", "T1").unwrap();

        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2);
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(TaskGraph::parse_duration(&Some("3-4h".to_string())), 3.0);
        assert_eq!(TaskGraph::parse_duration(&Some("2h".to_string())), 2.0);
        assert_eq!(TaskGraph::parse_duration(&Some("10-12h".to_string())), 10.0);
        assert_eq!(TaskGraph::parse_duration(&None), 1.0);
    }
}
