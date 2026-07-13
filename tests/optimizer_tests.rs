//! Tests for production optimization algorithms.

use aniimax::data::load_all_data;
use aniimax::models::{FacilityCounts, ModuleLevels, PlanStep, PlanStepStatus};
use aniimax::optimizer::{
    calculate_efficiencies, find_best_production_path, find_production_plan, time_to_reach_goal,
};
use std::path::Path;

fn default_facility_counts() -> FacilityCounts {
    FacilityCounts::from_pairs(&[
        ("Farmland", 1, 3),
        ("Woodland", 1, 3),
        ("Mineral Pile", 1, 3),
        ("Carousel Mill", 1, 3),
        ("Jukebox Dryer", 1, 3),
        ("Crafting Table", 1, 3),
        ("Nimbus Bed", 1, 1),
    ])
}

fn default_module_levels() -> ModuleLevels {
    ModuleLevels::default()
}

#[test]
fn test_calculate_efficiencies_coins() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = default_facility_counts();
    let modules = default_module_levels();

    let efficiencies = calculate_efficiencies(&items, "coins", &counts, &modules);

    assert!(!efficiencies.is_empty(), "Should find some coin-producing items");

    for eff in &efficiencies {
        assert_eq!(eff.item.sell_currency, "coins");
        assert!(eff.profit_per_second >= 0.0, "Profit per second should be non-negative");
        assert!(eff.total_time_per_unit > 0.0, "Total time should be positive");
    }
}

#[test]
fn test_calculate_efficiencies_bud_tickets() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = default_facility_counts();
    // Bud Ticket items ("Premium"/"Advanced" recipes) are all module-gated, so unlock
    // Crafting Module to get at least one (premium_wood_sculpture) into the results.
    let modules = ModuleLevels {
        crafting_module: 1,
        ..ModuleLevels::default()
    };

    let efficiencies = calculate_efficiencies(&items, "bud_tickets", &counts, &modules);

    assert!(
        !efficiencies.is_empty(),
        "Should find some Bud Ticket-producing items with Crafting Module unlocked"
    );

    for eff in &efficiencies {
        assert_eq!(eff.item.sell_currency, "bud_tickets");
    }
}

#[test]
fn test_calculate_efficiencies_filters_by_level() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let modules = default_module_levels();

    // Level 1 only
    let counts_level_1 = FacilityCounts::from_pairs(&[
        ("Farmland", 1, 1),
        ("Woodland", 1, 1),
        ("Mineral Pile", 1, 1),
        ("Carousel Mill", 1, 1),
        ("Jukebox Dryer", 1, 1),
        ("Crafting Table", 1, 1),
        ("Nimbus Bed", 0, 1),
    ]);

    // Level 3 for all
    let counts_level_3 = FacilityCounts::from_pairs(&[
        ("Farmland", 1, 3),
        ("Woodland", 1, 3),
        ("Mineral Pile", 1, 3),
        ("Carousel Mill", 1, 3),
        ("Jukebox Dryer", 1, 3),
        ("Crafting Table", 1, 3),
        ("Nimbus Bed", 1, 1),
    ]);

    let eff_level_1 = calculate_efficiencies(&items, "coins", &counts_level_1, &modules);
    let eff_level_3 = calculate_efficiencies(&items, "coins", &counts_level_3, &modules);

    // Higher level should have at least as many options
    assert!(
        eff_level_3.len() >= eff_level_1.len(),
        "Higher level should unlock more or equal items"
    );

    // Level 1 efficiencies should only contain level 1 items
    for eff in &eff_level_1 {
        assert_eq!(
            eff.item.facility_level, 1,
            "Level 1 counts should only show level 1 items"
        );
    }
}

#[test]
fn test_find_best_production_path() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = default_facility_counts();
    let modules = default_module_levels();

    let efficiencies = calculate_efficiencies(&items, "coins", &counts, &modules);
    let path = find_best_production_path(&efficiencies, 1000.0, false, 0.0, &counts);

    assert!(path.is_some(), "Should find a production path");

    let path = path.unwrap();
    assert!(path.total_profit >= 1000.0, "Should meet target profit");
    assert!(path.total_time > 0.0, "Should have positive time");
    assert!(!path.steps.is_empty(), "Should have at least one step");
    assert_eq!(path.currency, "coins");
}

#[test]
fn test_find_best_production_path_energy_optimization() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = default_facility_counts();
    let modules = default_module_levels();

    let efficiencies = calculate_efficiencies(&items, "coins", &counts, &modules);

    // Time optimization
    let path_time = find_best_production_path(&efficiencies, 1000.0, false, 0.0, &counts);

    // Energy optimization
    let path_energy = find_best_production_path(&efficiencies, 1000.0, true, 0.0, &counts);

    assert!(path_time.is_some());
    assert!(path_energy.is_some());

    // Both should meet the target
    assert!(path_time.unwrap().total_profit >= 1000.0);
    assert!(path_energy.unwrap().total_profit >= 1000.0);
}

#[test]
fn test_empty_efficiencies() {
    let efficiencies = vec![];
    let counts = default_facility_counts();

    let path = find_best_production_path(&efficiencies, 1000.0, false, 0.0, &counts);

    assert!(path.is_none(), "Should return None for empty efficiencies");
}

#[test]
fn test_parallel_production_increases_efficiency() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }

    let items = load_all_data(data_dir).expect("Failed to load data");
    let modules = default_module_levels();

    // Single facility
    let counts_single = FacilityCounts::from_pairs(&[
        ("Farmland", 1, 3),
        ("Woodland", 1, 3),
        ("Mineral Pile", 1, 3),
        ("Carousel Mill", 1, 3),
        ("Jukebox Dryer", 1, 3),
        ("Crafting Table", 1, 3),
        ("Nimbus Bed", 1, 1),
    ]);

    // Multiple facilities
    let counts_multi = FacilityCounts::from_pairs(&[
        ("Farmland", 4, 3),
        ("Woodland", 2, 3),
        ("Mineral Pile", 2, 3),
        ("Carousel Mill", 2, 3),
        ("Jukebox Dryer", 2, 3),
        ("Crafting Table", 2, 3),
        ("Nimbus Bed", 1, 1),
    ]);

    let eff_single = calculate_efficiencies(&items, "coins", &counts_single, &modules);
    let eff_multi = calculate_efficiencies(&items, "coins", &counts_multi, &modules);

    let path_single =
        find_best_production_path(&eff_single, 5000.0, false, 0.0, &counts_single);
    let path_multi = find_best_production_path(&eff_multi, 5000.0, false, 0.0, &counts_multi);

    assert!(path_single.is_some());
    assert!(path_multi.is_some());

    // Multiple facilities should complete faster or equal
    assert!(
        path_multi.unwrap().total_time <= path_single.unwrap().total_time,
        "Multiple facilities should be faster or equal"
    );
}

// `find_coin_plan` previously had no direct test coverage at all — every fix to it this session
// was verified live in the browser by hand. These lock in the two real bugs found and fixed via
// the LP rewrite (see BETA_NOTES.md): a shared root raw material being double-counted across
// sibling branches, and idle processing capacity never being credited to a second item.

#[test]
fn test_find_coin_plan_shares_root_raw_material_across_branches() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    // soy_sauce (Bouncy Brew Keg) and tofu (Carousel Mill) both need 8 soybean/batch from the
    // same Farmland — deliberately no Woodland/Jukebox Dryer/Mineral Pile so no other Claw Game
    // Cooker recipe can compete with soy_sauce_tofu for Farmland/Claw Game Cooker.
    let counts = FacilityCounts::from_pairs(&[
        ("Farmland", 20, 3),
        ("Bouncy Brew Keg", 1, 1),
        ("Carousel Mill", 2, 3),
        ("Claw Game Cooker", 1, 2),
    ]);
    let modules = ModuleLevels::default();

    let plan = find_production_plan(&items, "coins", &counts, &modules)
        .expect("plan should be feasible");
    let result = time_to_reach_goal(&plan, 1_000_000.0, 0.0).expect("goal should be reachable");

    let soy_sauce_tofu = result
        .products
        .iter()
        .find(|p| p.item_name == "soy_sauce_tofu")
        .expect("soy_sauce_tofu should be in the plan");

    // Correct rate (accounting for the shared 20-Farmland soybean supply) is ~6.1 coins/sec. A
    // per-branch calculation that lets each of soy_sauce/tofu assume exclusive access to all 20
    // Farmland independently would double that to ~12.2 — assert well below that regression
    // value, not just "some positive number".
    assert!(
        soy_sauce_tofu.rate_per_second > 4.0 && soy_sauce_tofu.rate_per_second < 8.0,
        "soy_sauce_tofu rate {} suggests the shared-soybean double-count regressed",
        soy_sauce_tofu.rate_per_second
    );
}

#[test]
fn test_find_coin_plan_processor_contention_dedicates_to_one_recipe_not_both() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    // Joy Wheel Loom (1 owned unit) can host both palm_fabric (Woodland-limited) and
    // cotton_fabric (Farmland-fed) — but a processor facility can only ever be "set and left" on
    // ONE recipe at a time, not switched between two to hit some fractional time-share. Regression
    // test for exactly that bug: an earlier pass let a single owned unit be split across both
    // recipes (22%/3%-style), which isn't something a player can actually execute in-game.
    let counts = FacilityCounts::from_pairs(&[
        ("Woodland", 10, 3),
        ("Farmland", 20, 2),
        ("Joy Wheel Loom", 1, 4),
    ]);
    let modules = ModuleLevels::default();

    let plan = find_production_plan(&items, "coins", &counts, &modules)
        .expect("plan should be feasible");

    let produced: Vec<&str> = plan
        .income_streams
        .iter()
        .filter(|p| p.item_name == "palm_fabric" || p.item_name == "cotton_fabric")
        .map(|p| p.item_name.as_str())
        .collect();
    assert_eq!(
        produced.len(),
        1,
        "Joy Wheel Loom should be dedicated to exactly one of palm_fabric/cotton_fabric, not \
         both, got: {:?}",
        produced
    );

    // No processor row should ever describe a time-share percentage — every processor row is
    // either a whole dedicated unit or explicitly idle.
    let jwl_steps: Vec<&PlanStep> =
        plan.coin_items.iter().filter(|s| s.facility == "Joy Wheel Loom").collect();
    assert!(
        jwl_steps.iter().all(|s| !s.reason.contains("% of the time")),
        "Joy Wheel Loom should never describe a time-share percentage, got: {:?}",
        jwl_steps
    );
    assert!(
        jwl_steps.iter().any(|s| s.status == PlanStepStatus::Producing),
        "Joy Wheel Loom should be producing its chosen recipe, not sitting fully idle"
    );

    // The recipe that lost the Joy Wheel Loom slot shouldn't just vanish — its raw material
    // capacity (Farmland or Woodland) should be reallocated to something else, not idle.
    let farmland_step = plan
        .coin_items
        .iter()
        .find(|s| s.facility == "Farmland" && s.status == PlanStepStatus::Producing);
    let woodland_step = plan
        .coin_items
        .iter()
        .find(|s| s.facility == "Woodland" && s.status == PlanStepStatus::Producing);
    assert!(
        farmland_step.is_some() && woodland_step.is_some(),
        "both Farmland and Woodland should still be producing something even though only one \
         of palm_fabric/cotton_fabric won the Joy Wheel Loom slot"
    );
}

// A processor facility with exactly ONE profitable contributor previously took a shortcut that
// reported the full owned count as dedicated to it, skipping the whole-unit/idle computation the
// multi-contributor path already did correctly. With 2 owned Crafting Tables and a single trickle
// of pearl (from 1 Tidewhisper Sandcastle) needing well under one unit's worth of capacity, only 1
// Crafting Table should show as producing pearl_necklace, with the other genuinely idle.
#[test]
fn test_find_coin_plan_solo_processor_contributor_reports_true_need_not_full_owned_count() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = FacilityCounts::from_pairs(&[
        ("Farmland", 24, 4),
        ("Woodland", 12, 3),
        ("Mineral Pile", 6, 3),
        ("Nimbus Bed", 1, 1),
        ("Grass Blossom Mat", 1, 1),
        ("Starfall Hammock", 1, 1),
        ("Tidewhisper Sandcastle", 1, 1),
        ("Carousel Mill", 2, 3),
        ("Phonolfactory Table", 1, 2),
        ("Bouncy Brew Keg", 1, 2),
        ("Crafting Table", 2, 3),
        ("Claw Game Cooker", 1, 2),
        ("Joy Wheel Loom", 1, 2),
        ("Jukebox Dryer", 2, 3),
    ]);
    let modules = ModuleLevels {
        ecological_module: 3,
        kitchen_module: 3,
        mineral_detector: 3,
        crafting_module: 3,
    };
    let plan = find_production_plan(&items, "coins", &counts, &modules).expect("plan should be feasible");

    let crafting_table_steps: Vec<&PlanStep> = plan
        .coin_items
        .iter()
        .filter(|s| s.facility == "Crafting Table")
        .collect();
    let producing: Vec<&&PlanStep> = crafting_table_steps
        .iter()
        .filter(|s| s.status == PlanStepStatus::Producing)
        .collect();
    let idle: Vec<&&PlanStep> = crafting_table_steps
        .iter()
        .filter(|s| s.status == PlanStepStatus::Idle)
        .collect();

    assert_eq!(
        producing.len(),
        1,
        "expected exactly one Crafting Table recipe, got: {:?}",
        crafting_table_steps
    );
    assert_eq!(
        producing[0].facility_count, 1,
        "a trickle of pearl from 1 Tidewhisper Sandcastle needs far less than 1 whole Crafting \
         Table, so only 1 of the 2 owned units should be dedicated, got: {:?}",
        crafting_table_steps
    );
    assert_eq!(
        idle.len(),
        1,
        "the second Crafting Table unit should show as genuinely idle, got: {:?}",
        crafting_table_steps
    );
    assert_eq!(idle[0].facility_count, 1);
}

#[test]
fn test_find_coin_plan_infeasible_with_no_facilities() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    // `FacilityCounts::get_count` returns 1 for any facility not explicitly listed (a "default to
    // owning one of everything" convenience for the CLI) — so genuine infeasibility requires
    // listing every facility at 0 explicitly, not just passing an empty slice.
    let counts = FacilityCounts::from_pairs(&[
        ("Farmland", 0, 1),
        ("Woodland", 0, 1),
        ("Mineral Pile", 0, 1),
        ("Nimbus Bed", 0, 1),
        ("Grass Blossom Mat", 0, 1),
        ("Starfall Hammock", 0, 1),
        ("Tidewhisper Sandcastle", 0, 1),
        ("Dewy House", 0, 1),
        ("Carousel Mill", 0, 1),
        ("Phonolfactory Table", 0, 1),
        ("Bouncy Brew Keg", 0, 1),
        ("Crafting Table", 0, 1),
        ("Claw Game Cooker", 0, 1),
        ("Joy Wheel Loom", 0, 1),
        ("Jukebox Dryer", 0, 1),
    ]);
    let modules = ModuleLevels::default();

    let plan = find_production_plan(&items, "coins", &counts, &modules);
    assert!(plan.is_none(), "no owned facilities should be infeasible");
}

#[test]
fn test_find_coin_plan_never_produces_via_unowned_intermediate_facility() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    // soy_sauce_tofu's only source of soy_sauce is Bouncy Brew Keg. Owning 0 of it must make
    // soy_sauce_tofu completely unproducible, even though every OTHER facility its chain touches
    // (Farmland for soybean, Carousel Mill for tofu, Claw Game Cooker for final assembly) is
    // owned. Regression test: `solve_facility_allocation` used to skip adding an LP constraint
    // entirely for any facility with 0 capacity, instead of constraining it to exactly 0 — which
    // let the solver treat "you own none of this" as "unlimited supply of it" for any facility
    // that only shows up as an INTERMEDIATE step in a chain (a root-facility check elsewhere
    // already caught the case where the item's own directly-owning facility is unowned, which is
    // why this bug was specific to intermediate facilities like Bouncy Brew Keg here).
    let counts = FacilityCounts::from_pairs(&[
        ("Farmland", 20, 3),
        ("Woodland", 0, 1),
        ("Mineral Pile", 0, 1),
        ("Nimbus Bed", 0, 1),
        ("Grass Blossom Mat", 0, 1),
        ("Starfall Hammock", 0, 1),
        ("Tidewhisper Sandcastle", 0, 1),
        ("Dewy House", 0, 1),
        ("Carousel Mill", 2, 3),
        ("Phonolfactory Table", 0, 1),
        ("Bouncy Brew Keg", 0, 1),
        ("Crafting Table", 0, 1),
        ("Claw Game Cooker", 1, 2),
        ("Joy Wheel Loom", 0, 1),
        ("Jukebox Dryer", 0, 1),
    ]);
    let modules = ModuleLevels::default();

    let plan = find_production_plan(&items, "coins", &counts, &modules)
        .expect("plan should still be feasible via some other item");
    let result = time_to_reach_goal(&plan, 600_000.0, 0.0).expect("goal should be reachable");

    assert!(
        !result.products.iter().any(|p| p.item_name == "soy_sauce_tofu"),
        "soy_sauce_tofu must not be produced with 0 Bouncy Brew Keg, got products: {:?}",
        result.products.iter().map(|p| &p.item_name).collect::<Vec<_>>()
    );
    assert!(
        !plan
            .coin_items
            .iter()
            .any(|s| s.item_name.as_deref() == Some("soy_sauce_tofu")),
        "soy_sauce_tofu must not appear in the facility plan with 0 Bouncy Brew Keg"
    );
}

#[test]
fn test_find_coin_plan_target_already_met() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    let counts = default_facility_counts();
    let modules = default_module_levels();

    let plan = find_production_plan(&items, "coins", &counts, &modules)
        .expect("plan should be feasible");
    let result = time_to_reach_goal(&plan, 1000.0, 5000.0)
        .expect("already-met target should be trivially feasible");
    assert_eq!(result.total_time, 0.0);
    assert_eq!(result.amount_produced, 0.0);
}

#[test]
fn test_find_production_plan_bud_tickets_end_to_end() {
    let data_dir = Path::new("data");
    if !data_dir.exists() {
        return;
    }
    let items = load_all_data(data_dir).expect("Failed to load data");
    // advanced_soy_sauce_tofu (Claw Game Cooker: soy_sauce + tofu, kitchen_module:3) sells for
    // Bud Tickets only — needs the same soy_sauce/tofu supply chain as the coins-only
    // soy_sauce_tofu, so this doubles as a check that currency filtering happens without
    // otherwise touching the LP/chain-resolution logic.
    let counts = FacilityCounts::from_pairs(&[
        ("Farmland", 20, 3),
        ("Bouncy Brew Keg", 1, 1),
        ("Carousel Mill", 2, 3),
        ("Claw Game Cooker", 1, 2),
    ]);
    let modules = ModuleLevels {
        kitchen_module: 3,
        ..ModuleLevels::default()
    };

    let bud_ticket_plan = find_production_plan(&items, "bud_tickets", &counts, &modules)
        .expect("bud_tickets plan should be feasible");
    assert_eq!(bud_ticket_plan.currency, "bud_tickets");
    assert!(
        bud_ticket_plan
            .income_streams
            .iter()
            .any(|p| p.item_name == "advanced_soy_sauce_tofu"),
        "advanced_soy_sauce_tofu should be produced when optimizing for bud_tickets, got: {:?}",
        bud_ticket_plan.income_streams.iter().map(|p| &p.item_name).collect::<Vec<_>>()
    );
    assert!(
        !bud_ticket_plan.income_streams.iter().any(|p| p.item_name == "soy_sauce_tofu"),
        "coins-only soy_sauce_tofu should not appear when optimizing for bud_tickets"
    );

    let coin_plan = find_production_plan(&items, "coins", &counts, &modules)
        .expect("coins plan should be feasible");
    assert_eq!(coin_plan.currency, "coins");
    assert!(
        !coin_plan.income_streams.iter().any(|p| p.item_name == "advanced_soy_sauce_tofu"),
        "advanced_soy_sauce_tofu should not appear when optimizing for coins"
    );
}
