// Quick test script to debug parser
use std::path::PathBuf;

fn main() {
    let mut plan_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    plan_path.push("claudedocs");
    plan_path.push("PLAN_PHASES_F-15.md");

    println!("Parsing: {:?}", plan_path);

    let result = ccboard_core::parsers::PlanParser::parse_file(&plan_path).unwrap();
    let plan = result.expect("Expected Some(PlanFile)");

    println!("\n=== Metadata ===");
    println!("Title: {}", plan.metadata.title);
    println!("Status: {}", plan.metadata.status);
    println!("Version: {:?}", plan.metadata.version);

    println!("\n=== Phases ({}) ===", plan.phases.len());
    for phase in &plan.phases {
        println!(
            "- Phase {}: {} ({} tasks, priority: {:?})",
            phase.id,
            phase.title,
            phase.tasks.len(),
            phase.priority
        );
    }
}
