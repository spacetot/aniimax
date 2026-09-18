//! Tests for the exact planner (`aniimax::exact`). Each scenario checks that the plan is proven
//! optimal, passes the independent re-check, never earns less than the heuristic planner, and,
//! where worked out by hand, earns exactly what the arithmetic in the comments says.

use aniimax::data::{load_all_data, load_aniimo_requirements};
use aniimax::exact::{check_plan, solve_exact, to_production_plan, ExactPlan, Goal};
use aniimax::models::{AniimoSetup, FacilityCounts, ModuleLevels, ProductionItem};
use aniimax::optimizer::find_production_plan;
use std::path::Path;
use std::time::Duration;

fn load_items() -> Option<Vec<ProductionItem>> {
    let data_dir = Path::new("data");
    data_dir.exists().then(|| load_all_data(data_dir).expect("Failed to load data"))
}

/// Solves `counts`/`modules` exactly and checks what every scenario should hold.
fn solve_and_check(items: &[ProductionItem], counts: &FacilityCounts, modules: &ModuleLevels) -> ExactPlan {
    let plan = solve_exact(items, "coins", counts, modules, Goal::Earn { floors: &[] }, Some(Duration::from_secs(60)), None)
        .expect("exact plan");
    assert!(plan.proven_optimal, "not proven optimal: {plan:?}");
    let recomputed = check_plan(&plan, items, "coins", counts, modules).expect("plan passes its re-check");
    assert!((recomputed - plan.rate_per_second).abs() < 1e-6);
    let heuristic = find_production_plan(items, "coins", counts, modules, false).map_or(0.0, |p| p.rate_per_second);
    assert!(
        plan.rate_per_second >= heuristic - 1e-6,
        "exact {} below heuristic {heuristic}",
        plan.rate_per_second
    );
    plan
}

// Six Farmland at level 2 with nothing else: wheat (5 per 240s, sells for 1) earns 6 x 5/240 =
// 0.125/sec; potato (2 per 640s, sells for 8, seed 1) earns 6 x 15/640 = 0.1406; quick_wheat needs
// Ecological Module 1, which isn't owned. Potato wins.
#[test]
fn exact_picks_the_best_crop_on_its_own() {
    let Some(items) = load_items() else { return };
    let counts = FacilityCounts::only(&[("Farmland", 6, 2)]);
    let plan = solve_and_check(&items, &counts, &ModuleLevels::default());
    assert!((plan.rate_per_second - 6.0 * 15.0 / 640.0).abs() < 1e-9, "got {}", plan.rate_per_second);
    assert_eq!(plan.units.get("potato"), Some(&6));
}

// rice_drink (Carousel Mill) needs milled_rice made at a Carousel Mill too; with two Mills, one
// makes each. 3 rice plots -> 0.000625 rice_drink/sec x (1860 - 2 x 12 seed) = 1.1475; 4 Wells with
// level-1 Aniimo make 0.014222 fresh_water/sec, rice_drink uses 0.01 and the rest sells:
// 0.004222 x 46 = 0.19422. Total 1.34172.
#[test]
fn exact_matches_the_hand_worked_rice_drink_plan() {
    let Some(items) = load_items() else { return };
    let counts = FacilityCounts::only(&[("Farmland", 3, 3), ("Well", 4, 2), ("Carousel Mill", 2, 4)]);
    let plan = solve_and_check(&items, &counts, &ModuleLevels::default());
    let expected = 3.0 * 18.0 / 2400.0 / 18.0 / 2.0 * (1860.0 - 24.0) + (4.0 * 8.0 / 2250.0 - 0.01) * 46.0;
    assert!((plan.rate_per_second - expected).abs() < 1e-6, "got {}, expected {expected}", plan.rate_per_second);
    assert_eq!(plan.units.get("rice_drink"), Some(&1));
    assert_eq!(plan.units.get("milled_rice"), Some(&1));
}

// A mid-game setup with an Aniimo setup applied and a Cooling Unit, so environment coverage and
// whole processor units both matter: the exact plan is proven, re-checks, and beats or ties the
// heuristic planner; its rows cover every owned unit exactly once.
#[test]
fn exact_handles_environments_and_aniimo_speeds() {
    let Some(mut items) = load_items() else { return };
    load_aniimo_requirements(Path::new("data")).unwrap().apply(AniimoSetup::Best, &mut items);
    let counts = FacilityCounts::only(&[
        ("Farmland", 14, 4),
        ("Woodland", 8, 3),
        ("Mine", 4, 2),
        ("Well", 1, 1),
        ("Cooling Unit", 1, 1),
        ("Carousel Mill", 1, 2),
        ("Crafting Table", 1, 3),
        ("Jukebox Dryer", 1, 3),
        ("Claw Game Cooker", 1, 3),
        ("Phonolfactory Table", 1, 2),
        ("Dewy House", 1, 1),
        ("Tidewhisper Sandcastle", 1, 1),
    ]);
    let modules = ModuleLevels { ecological_module: 2, kitchen_module: 2, resource_detector: 1, crafting_module: 2 };
    let plan = solve_and_check(&items, &counts, &modules);
    let shown = to_production_plan(&plan, &items, "coins", &counts);
    for facility in ["Farmland", "Woodland", "Mine", "Crafting Table", "Jukebox Dryer"] {
        let rows: u32 = shown.coin_items.iter().filter(|s| s.facility == facility).map(|s| s.facility_count).sum();
        assert_eq!(rows, counts.get_count(facility), "{facility} rows: {:?}", shown.coin_items);
    }
    let streams: f64 = shown.income_streams.iter().map(|s| s.rate_per_second).sum();
    assert!((streams - plan.rate_per_second).abs() < 1e-6, "income streams add to {streams}");
}

// "Prioritize byproducts": the most Wood Blocks the Woodland can make is found first, and the
// earning plan must still make that much.
#[test]
fn exact_keeps_prioritized_byproducts_at_their_maximum() {
    let Some(items) = load_items() else { return };
    let counts = FacilityCounts::only(&[("Woodland", 4, 2), ("Jukebox Dryer", 1, 2)]);
    let modules = ModuleLevels::default();
    let most = solve_exact(&items, "coins", &counts, &modules, Goal::MostOf("Wood Blocks"), None, None).unwrap();
    assert!(most.proven_optimal && most.rate_per_second > 0.0);
    let floors = vec![("Wood Blocks".to_string(), most.rate_per_second)];
    let plan = solve_exact(&items, "coins", &counts, &modules, Goal::Earn { floors: &floors }, None, None).unwrap();
    assert!(plan.proven_optimal);
    let made: f64 = plan
        .recipe_rates
        .iter()
        .filter_map(|(name, rate)| {
            let item = items.iter().find(|i| &i.name == name)?;
            item.byproduct.as_ref().filter(|(r, _)| r == "Wood Blocks").map(|(_, amount)| rate * *amount as f64)
        })
        .sum();
    assert!(made >= most.rate_per_second * (1.0 - 1e-5), "made {made}, most {}", most.rate_per_second);
    let unfloored = solve_exact(&items, "coins", &counts, &modules, Goal::Earn { floors: &[] }, None, None).unwrap();
    assert!(plan.rate_per_second <= unfloored.rate_per_second + 1e-9);
}
