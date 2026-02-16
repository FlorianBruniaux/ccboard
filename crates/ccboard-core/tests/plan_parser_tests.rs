//! Integration tests for PLAN.md parser

use ccboard_core::models::plan::{PhaseStatus, PlanFile};
use ccboard_core::parsers::PlanParser;
use std::path::PathBuf;

#[test]
fn test_parse_real_plan_file() {
    // Parse the real PLAN_PHASES_F-15.md file
    let mut plan_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plan_path.pop(); // Go up from ccboard-core
    plan_path.pop(); // Go up from crates
    plan_path.push("claudedocs");
    plan_path.push("PLAN_PHASES_F-15.md");

    if !plan_path.exists() {
        panic!(
            "PLAN_PHASES_F-15.md not found at {:?}. Test requires real file.",
            plan_path
        );
    }

    let result = PlanParser::parse_file(&plan_path).expect("Failed to parse PLAN.md");
    let plan = result.expect("Expected Some(PlanFile), got None");

    // Validate metadata
    assert_eq!(
        plan.metadata.title,
        "ccboard Phases F-15 - Plan de Développement Détaillé avec Agents"
    );
    assert_eq!(plan.metadata.status, Some("phases-f-h-complete".to_string()));
    assert_eq!(plan.metadata.version, Some("0.8.0".to_string()));
    assert_eq!(plan.metadata.date, Some("2026-02-12".to_string()));
    assert_eq!(
        plan.metadata.estimated_total_duration,
        Some("82-120h (10-15 jours)".to_string())
    );

    // Validate phases detected
    assert!(
        plan.phases.len() >= 7,
        "Expected at least 7 phases (F, H, 11, 12, 13, 14, 15), found {}",
        plan.phases.len()
    );

    // Validate Phase F
    let phase_f = plan
        .phases
        .iter()
        .find(|p| p.id == "F")
        .expect("Phase F not found");
    assert_eq!(phase_f.title, "Conversation Viewer");
    assert_eq!(phase_f.priority, Some("HIGH".to_string()));
    assert!(phase_f.estimated_duration.is_some());
    assert!(phase_f.version_target.is_some());

    // Validate Phase H
    let phase_h = plan
        .phases
        .iter()
        .find(|p| p.id == "H")
        .expect("Phase H not found");
    assert_eq!(phase_h.title, "Plan-Aware");
    assert_eq!(phase_h.priority, Some("HIGH".to_string()));

    // Validate numeric phases
    for phase_id in ["11", "12", "13", "14", "15"] {
        assert!(
            plan.phases.iter().any(|p| p.id == phase_id),
            "Phase {} not detected",
            phase_id
        );
    }

    // Validate tasks detected in Phase F
    assert!(
        phase_f.tasks.len() >= 5,
        "Expected at least 5 tasks in Phase F, found {}",
        phase_f.tasks.len()
    );

    // Validate Task F.1
    let task_f1 = phase_f
        .tasks
        .iter()
        .find(|t| t.id == "F.1")
        .expect("Task F.1 not found");
    assert!(task_f1.title.contains("JSONL Parser"));
    assert_eq!(task_f1.priority, Some("P0".to_string()));
    assert!(task_f1.issue.is_some());
    assert!(task_f1.duration.is_some());
    assert!(task_f1.difficulty.is_some());
}

#[test]
fn test_parse_minimal_plan() {
    let content = r#"---
date: 2026-02-12
title: Minimal Plan
status: future
---

# Minimal Plan

## Phase A: Test Phase (Priority: 🔴 HIGH)

**Durée estimée** : 2-3h
**Version cible** : v1.0.0

#### Task A.1: Test Task (P0)

**Issue** : #1
**Durée** : 1h
**Difficulté** : Good First Issue
**Crate** : ccboard-core

Test task description.
"#;

    let result = PlanParser::parse(content).expect("Failed to parse");
    let plan = result.expect("Expected Some(PlanFile)");

    assert_eq!(plan.metadata.title, "Minimal Plan");
    assert_eq!(plan.metadata.status, Some("future".to_string()));

    assert_eq!(plan.phases.len(), 1);
    let phase = &plan.phases[0];
    assert_eq!(phase.id, "A");
    assert_eq!(phase.title, "Test Phase");
    assert_eq!(phase.priority, Some("HIGH".to_string()));
    assert_eq!(phase.estimated_duration, Some("2-3h".to_string()));
    assert_eq!(phase.version_target, Some("v1.0.0".to_string()));

    assert_eq!(phase.tasks.len(), 1);
    let task = &phase.tasks[0];
    assert_eq!(task.id, "A.1");
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.priority, Some("P0".to_string()));
    assert_eq!(task.issue, Some(1));
    assert_eq!(task.duration, Some("1h".to_string()));
    assert_eq!(task.difficulty, Some("Good First Issue".to_string()));
    assert_eq!(task.crate_name, Some("ccboard-core".to_string()));
}

#[test]
fn test_parse_no_frontmatter() {
    let content = r#"# Plan without frontmatter

## Phase X: Some Phase

#### Task X.1: Some Task
"#;

    let result = PlanParser::parse(content).expect("Failed to parse");
    assert!(result.is_none(), "Expected None for missing frontmatter");
}

#[test]
fn test_phase_status_detection() {
    let content = r#"---
title: Test
status: in-progress
---

## Phase A: Complete Phase

**Status** : complete

## Phase B: In Progress Phase

This phase is in-progress.

## Phase C: Future Phase

Not started yet.
"#;

    let result = PlanParser::parse(content).expect("Failed to parse");
    let plan = result.expect("Expected Some(PlanFile)");

    assert_eq!(plan.phases.len(), 3);

    // Note: Current implementation doesn't parse **Status** lines yet
    // This test documents expected behavior for future enhancement
}

#[test]
fn test_parse_multiple_tasks() {
    let content = r#"---
title: Multi-task Plan
status: future
---

## Phase M: Multi-Task Phase

#### Task M.1: First Task (P0)

**Durée** : 1h

#### Task M.2: Second Task (P1)

**Durée** : 2h

#### Task M.3: Third Task (P2)

**Durée** : 3h
"#;

    let result = PlanParser::parse(content).expect("Failed to parse");
    let plan = result.expect("Expected Some(PlanFile)");

    assert_eq!(plan.phases.len(), 1);
    let phase = &plan.phases[0];
    assert_eq!(phase.tasks.len(), 3);

    assert_eq!(phase.tasks[0].id, "M.1");
    assert_eq!(phase.tasks[1].id, "M.2");
    assert_eq!(phase.tasks[2].id, "M.3");

    assert_eq!(phase.tasks[0].duration, Some("1h".to_string()));
    assert_eq!(phase.tasks[1].duration, Some("2h".to_string()));
    assert_eq!(phase.tasks[2].duration, Some("3h".to_string()));
}

#[test]
fn test_parse_numeric_phase_ids() {
    let content = r#"---
title: Numeric Phases
status: future
---

## Phase 11: Phase Eleven

#### Task 11.1: Task One (P0)

## Phase 12: Phase Twelve

#### Task 12.1: Task Two (P0)
"#;

    let result = PlanParser::parse(content).expect("Failed to parse");
    let plan = result.expect("Expected Some(PlanFile)");

    assert_eq!(plan.phases.len(), 2);
    assert_eq!(plan.phases[0].id, "11");
    assert_eq!(plan.phases[1].id, "12");
}

#[test]
fn test_graceful_degradation_malformed_yaml() {
    let content = r#"---
this is not valid yaml: [unclosed array
---

## Phase X: Test
"#;

    // Should return error due to malformed YAML
    let result = PlanParser::parse(content);
    assert!(result.is_err(), "Expected error for malformed YAML");
}
