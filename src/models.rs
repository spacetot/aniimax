//! Data models and structures for Aniimax.
//!
//! This module contains all the core data structures used throughout the application,
//! including production items, efficiency calculations, and production paths.

use serde::Deserialize;
use std::collections::HashSet;

/// Represents a single production item that can be produced in the game.
///
/// This includes both raw materials (from Farmland, Woodland, Mineral Pile)
/// and processed items (from various processing facilities).
///
/// # Example
///
/// ```
/// use aniimax::models::ProductionItem;
///
/// let wheat = ProductionItem {
///     name: "wheat".to_string(),
///     facility: "Farmland".to_string(),
///     raw_materials: None,
///     required_amount: None,
///     cost: Some(0.0),
///     sell_currency: "coins".to_string(),
///     sell_value: 1.0,
///     production_time: 90.0,
///     yield_amount: 10,
///     energy: Some(809.0),
///     facility_level: 1,
///     module_requirement: None,
///     requires_fertilizer: false,
///     workload: None,
///     byproduct: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ProductionItem {
    /// The name of the item (e.g., "wheat", "potato_chips")
    pub name: String,
    /// The facility where this item is produced (e.g., "Farmland", "Carousel Mill")
    pub facility: String,
    /// The raw materials required for processing (None for raw materials, can be multiple)
    pub raw_materials: Option<Vec<String>>,
    /// The amount of each raw material required per production (parallel to raw_materials)
    pub required_amount: Option<Vec<u32>>,
    /// The cost to plant/start production (for raw materials)
    pub cost: Option<f64>,
    /// The currency received when selling ("coins" or "bud_tickets")
    pub sell_currency: String,
    /// The value received per unit when selling
    pub sell_value: f64,
    /// Time in seconds to complete one production cycle
    pub production_time: f64,
    /// Number of items yielded per production cycle
    pub yield_amount: u32,
    /// Energy gained when this item is consumed (None = cannot be consumed for energy)
    pub energy: Option<f64>,
    /// Minimum facility level required to produce this item
    pub facility_level: u32,
    /// Module requirement: (module_name, required_level) - None if no module needed
    pub module_requirement: Option<(String, u32)>,
    /// Whether this item requires fertilizer to produce
    pub requires_fertilizer: bool,
    /// Workload stat (new-beta Aniimo-dispatch facilities only). Informational — `production_time`
    /// already reflects the derived time at 100% efficiency; see [`WORKLOAD_RATE_ESTIMATE`].
    pub workload: Option<f64>,
    /// Secondary byproduct yielded alongside the main product: (resource_name, amount).
    /// E.g. Woodland yields Wood Blocks, Mineral Pile yields Mineral Sand. These are
    /// progression resources (Homeland/RV upgrades), not currency, so they are not folded
    /// into the profit optimizer — informational only.
    pub byproduct: Option<(String, u32)>,
}

/// Calibration constant for converting `workload` (any workload-driven, Aniimo-dispatch
/// facility) into an estimated production time at 100% Aniimo efficiency, in
/// workload-units-per-second. Applied universally across Mineral Pile, Nimbus Bed, Carousel
/// Mill, Crafting Table, and any other workload-based facility.
///
/// Derived from a single confirmed data point: Shell at Mineral Pile (workload 300) took 3m29s
/// (209s) at 100% efficiency, giving 300/209 ≈ 1.4354 workload/sec. This is our only *direct*
/// 100%-efficiency measurement (a second data point, Petals at Nimbus Bed, was only observed at
/// 140% efficiency and required assuming linear scaling to back out an implied 100%-rate — a
/// weaker estimate). We initially kept separate per-facility constants, but identical workload
/// "tier" values turning up across unrelated facilities (e.g. Wheatmeal at Carousel Mill and
/// Shell Ornament at Crafting Table both have workload 18) suggests workload may be a shared
/// game-wide unit rather than facility-specific, so this rate is now applied universally as the
/// best available estimate. Still provisional — revisit if a direct 100%-efficiency calibration
/// point for a different facility ever contradicts it.
pub const WORKLOAD_RATE_ESTIMATE: f64 = 300.0 / 209.0;

/// Efficiency metrics for an item when consumed for energy.
#[derive(Debug, Clone)]
pub struct EnergyItemEfficiency {
    /// The production item
    pub item: ProductionItem,
    /// Energy gained per second of production time
    pub energy_per_second: f64,
    /// Time to produce one batch
    pub time_per_batch: f64,
    /// Energy gained per batch when consumed
    pub energy_per_batch: f64,
    /// Cost (in coins) per batch
    pub cost_per_batch: f64,
}

/// Represents an optimized production path to achieve a target currency goal.
///
/// Contains the sequence of production steps, timing information,
/// and overall efficiency metrics.
#[derive(Debug, Clone)]
pub struct ProductionPath {
    /// Ordered list of production steps to execute
    pub steps: Vec<ProductionStep>,
    /// Total time required to complete all production (in seconds)
    pub total_time: f64,
    /// Startup time before steady-state production begins (max first-batch time across parallel chains)
    pub startup_time: f64,
    /// Total energy consumed (calculated as time * energy_cost_per_min / 60)
    pub total_energy: Option<f64>,
    /// Total profit generated
    pub total_profit: f64,
    /// The currency type being produced
    pub currency: String,
    /// Total number of items that will be produced for sale
    pub items_produced: u32,
    /// Whether this path is energy self-sufficient
    pub is_energy_self_sufficient: bool,
    /// Energy items produced for consumption (if self-sufficient)
    pub energy_items_produced: Option<u32>,
    /// Name of item used for energy (if self-sufficient)
    pub energy_item_name: Option<String>,
}

/// Represents a single step in a production path.
///
/// Each step describes what to produce, where, and in what quantity.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProductionStep {
    /// Name of the item to produce
    pub item_name: String,
    /// Facility where production occurs (includes count, e.g., "Farmland (x4)")
    pub facility: String,
    /// Number of production cycles to run
    pub quantity: u32,
    /// Time for this step (in seconds)
    pub time: f64,
    /// Energy consumed by this step
    pub energy: Option<f64>,
    /// Profit contribution from this step
    pub profit_contribution: f64,
    /// Chain ID for parallel production (steps with same ID run together)
    pub chain_id: Option<u32>,
    /// Optimal facility allocation: Vec<(material_name, batches_needed, facilities_to_allocate)>
    /// Shows how to split facilities when producing multiple materials to minimize time
    pub facility_allocation: Option<Vec<(String, u32, u32)>>,
}

/// What role a facility plays within a [`ProductionPlan`] — lets the result explain itself
/// instead of just listing an item name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStepStatus {
    /// Actively dedicated to producing `item_name`.
    Producing,
    /// Owned, but has nothing profitable to produce right now (e.g. its raw materials aren't
    /// being produced by anything, since the facility that would make them is dedicated
    /// elsewhere, or nothing is unlocked at its current level/module tier).
    NothingAvailable,
    /// Has a profitable item, but isn't needed for this plan.
    NotNeeded,
    /// Some of this facility's capacity is producing (see the sibling rows for the same
    /// `facility`), but this portion has no further profitable use and sits idle. For a grower
    /// facility this is unassigned whole-unit plots; for a processor facility it's spare
    /// dedicated-unit capacity left over after every contributor got its own whole unit (see
    /// `PlanStep::facility_count`).
    Idle,
}

/// A single row within a [`ProductionPlan`]'s facility-plan table — describes exactly one
/// facility producing exactly one item (or its idle/unused capacity, or having nothing
/// available). A facility that splits its capacity across multiple items gets multiple rows, one
/// per item, so every row is always single-product — never a joined "X + Y" description.
#[derive(Debug, Clone)]
pub struct PlanStep {
    /// Item this row describes. `None` when `status` isn't `Producing`.
    pub item_name: Option<String>,
    /// Facility producing it
    pub facility: String,
    /// Number of this facility's units dedicated to this row's item (or left idle). For a
    /// processor facility this is a whole dedicated unit whenever there's enough owned capacity
    /// for every contributor to get one (always true when it's the only contributor) — dedicating
    /// a whole unit never changes the achievable rate computed above, since a unit's throughput
    /// ceiling only ever exceeds its actual share of jointly-run time. Only when more distinct
    /// items want this facility than can each get a dedicated unit does this fall back to
    /// reporting the full owned count on every contending row, with `reason` stating each item's
    /// time share instead.
    pub facility_count: u32,
    /// What role this row plays in the plan
    pub status: PlanStepStatus,
    /// Human-readable explanation of `status`, for display
    pub reason: String,
}

/// A single income stream within a [`ProductionPlan`] — either a fully-selected item, or the
/// leftover-capacity portion of a facility that's split between feeding a recipe and selling
/// directly (section 36). Reported as "what actually got made," as opposed to [`PlanStep`] which
/// reports "what each facility does" — this is the item-level breakdown: how much of each thing,
/// its rate, and its total contribution to the plan.
///
/// Doubles as both the target-independent form (as found in `ProductionPlan.income_streams`,
/// where `total_units`/`total_value` are left at `0.0`) and the target-dependent form (as
/// returned in `GoalResult.products`, where those two fields are filled in) — see
/// `crate::optimizer::time_to_reach_goal`.
#[derive(Debug, Clone)]
pub struct PlanProduct {
    /// Name of the item produced/sold
    pub item_name: String,
    /// Facility where it's produced
    pub facility: String,
    /// Sell price per unit — lets a viewer verify `total_units * sell_value` against the
    /// reported worth by hand, since `total_units` gets floored for display (you can't sell a
    /// fractional item) while `total_value` below is the unrounded net-profit figure.
    pub sell_value: f64,
    /// Currency units earned per second while this stream is active (net of ingredient costs)
    pub rate_per_second: f64,
    /// Units produced per second while active
    pub units_per_second: f64,
    /// Seconds before this stream's first output exists (0 if nothing is blocking it)
    pub lead_time: f64,
    /// Total units produced over the whole plan (0 until filled in by `time_to_reach_goal`)
    pub total_units: f64,
    /// Total currency earned from this item over the whole plan (net of ingredient costs,
    /// unrounded — the UI's gross "worth" column is computed from `sell_value *
    /// floor(total_units)` instead, so it reconciles with the whole-number amount actually
    /// shown). 0 until filled in by `time_to_reach_goal`.
    pub total_value: f64,
}

/// The provably-optimal simultaneous use of every owned facility for one currency ("coins" or
/// "bud_tickets") — target-independent: this is "what's the best I can do," computed before any
/// goal amount is known. See `crate::optimizer::find_production_plan` for the algorithm, and
/// `crate::optimizer::time_to_reach_goal` for turning this plan plus a goal amount into a
/// [`GoalResult`]. See `BETA_NOTES.md` section 23 (and the follow-ups in sections 27–29, 43-46)
/// for the full design writeup of the plan-solving side; this type originally also tracked Wood
/// Blocks/Mineral Sand progress toward a Homeland/RV upgrade, dropped in section 30 since those
/// are trivially obtained by expanding plots in-game and aren't worth optimizing for.
#[derive(Debug, Clone)]
pub struct ProductionPlan {
    /// The currency this plan was optimized for: `"coins"` or `"bud_tickets"`
    pub currency: String,
    /// Combined steady-state rate (currency units/sec) once every income stream's lead time has
    /// passed — the sum of `income_streams`' `rate_per_second`. The headline "your rate" number.
    pub rate_per_second: f64,
    /// One entry per item the plan actually produces, `total_units`/`total_value` left at `0.0`
    /// until `time_to_reach_goal` fills them in for a specific goal.
    pub income_streams: Vec<PlanProduct>,
    /// One entry per owned facility, each running its own best item simultaneously. `item_name`
    /// is `None` if that facility currently has nothing profitable to produce.
    pub coin_items: Vec<PlanStep>,
    /// Every grower-facility byproduct contribution (Wood Blocks/Mineral Sand — purely
    /// informational, not optimized for, section 38b), kept as separate `(resource_name,
    /// rate_per_second, lead_time)` triples rather than pre-summed by resource, since different
    /// contributions to the same resource can have different lead times — summed into totals by
    /// `time_to_reach_goal` once a plan's duration is known.
    pub byproduct_rates: Vec<(String, f64, f64)>,
}

/// Result of turning a [`ProductionPlan`] plus a goal amount into a concrete time-to-target — the
/// fastest way to reach a target currency balance given the current balance, using the plan's
/// already-computed rates (no facility-allocation re-solve).
#[derive(Debug, Clone)]
pub struct GoalResult {
    /// Total time (seconds) for the target to be met
    pub total_time: f64,
    /// Total currency produced over `total_time`
    pub amount_produced: f64,
    /// Item-level production breakdown — one entry per income stream that actually produced
    /// something before `total_time` elapsed, sorted by `total_value` descending. Replaces the
    /// coin-income-over-time chart (section 33, removed in section 37) with a table instead.
    pub products: Vec<PlanProduct>,
    /// Total Wood Blocks/Mineral Sand produced as a side effect over `total_time` — purely
    /// informational (section 38b). `(resource_name, total_amount)` pairs, omitting any resource
    /// with a zero total.
    pub byproducts: Vec<(String, f64)>,
}

/// Calculated efficiency metrics for a production item.
///
/// Used to compare and rank different production options.
#[derive(Debug, Clone)]
pub struct ProductionEfficiency {
    /// The production item being evaluated
    pub item: ProductionItem,
    /// Profit generated per second of production time
    pub profit_per_second: f64,
    /// Profit generated per unit of energy consumed
    pub profit_per_energy: Option<f64>,
    /// Total time to produce one unit (including raw material gathering)
    pub total_time_per_unit: f64,
    /// Total energy to produce one unit (including raw material gathering)
    pub total_energy_per_unit: Option<f64>,
    /// Name of required raw material (if any)
    pub requires_raw: Option<String>,
    /// Cost of raw materials per production
    pub raw_cost: f64,
    /// Facility that produces the raw material
    pub raw_facility: Option<String>,
    /// All facilities used in this production chain (including intermediate processing)
    pub all_facilities: HashSet<String>,
    /// Intermediate processing steps: Vec<(item_name, facility, required_amount_per_batch)>
    pub intermediate_steps: Vec<(String, String, u32)>,
    /// Time to produce the first batch (startup delay before steady-state)
    pub startup_time: f64,
    /// Effective profit per second considering parallel facility usage
    pub effective_profit_per_second: f64,
    /// Raw material details for optimal allocation: Vec<(name, amount_per_batch, time_per_batch)>
    pub raw_material_details: Option<Vec<(String, u32, f64)>>,
    /// Fertilizer batches needed per production batch (0 if no fertilizer required)
    pub fertilizer_per_batch: u32,
    /// Every FACILITY touched anywhere in this item's ingredient tree (including this item's own
    /// facility, and any intermediate processing steps, not just direct/root raw materials),
    /// paired with its accumulated utilization (batches/sec of whatever runs there, weighted by
    /// that item's own production time — see `optimizer::compute_resource_demand`) required per
    /// one batch/sec of this item, and which item name(s) are hosted there in this tree. Used
    /// both to compute `effective_profit_per_second` correctly when multiple branches (or
    /// multiple different items) share one facility — e.g. soy_sauce_tofu's soy_sauce and tofu
    /// both drawing from the same Farmland soybean supply — and to find leftover capacity on any
    /// touched facility, including intermediate processors like Carousel Mill, not just direct
    /// raw-material suppliers, whose owned count exceeds what this item's true bottleneck-limited
    /// rate actually needs.
    pub facility_demand: Vec<(String, f64, Vec<String>)>,
}

/// Tracks the number of each facility type available.
///
/// Multiple facilities of the same type allow for parallel production,
/// reducing overall production time.
///
/// Backed by a name → (count, level) map rather than fixed struct fields, so new facilities
/// (of which the new beta has many — see `BETA_NOTES.md`) can be added without touching this
/// struct's definition or any of its call sites.
///
/// # Example
///
/// ```
/// use aniimax::models::FacilityCounts;
///
/// let counts = FacilityCounts::from_pairs(&[
///     ("Farmland", 4, 2),      // 4 plots at level 2
///     ("Woodland", 2, 1),      // 2 plots at level 1
///     ("Mineral Pile", 1, 1),
/// ]);
///
/// assert_eq!(counts.get_count("Farmland"), 4);
/// assert_eq!(counts.get_level("Farmland"), 2);
/// // Facilities not set default to count/level 1 (matches old "unknown facility" behavior).
/// assert_eq!(counts.get_count("Carousel Mill"), 1);
/// ```
#[derive(Debug, Clone)]
pub struct FacilityCounts {
    facilities: std::collections::HashMap<String, (u32, u32)>,
    /// Level reported for any facility not explicitly `set()` (normally 1). Used by
    /// [`FacilityCounts::show_all_levels`] to report every facility as maximally unlocked.
    default_level: u32,
}

impl Default for FacilityCounts {
    fn default() -> Self {
        Self {
            facilities: std::collections::HashMap::new(),
            default_level: 1,
        }
    }
}

impl FacilityCounts {
    /// Creates an empty `FacilityCounts` (every facility defaults to count=1, level=1).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a `FacilityCounts` from a list of `(facility_name, count, level)` triples.
    pub fn from_pairs(pairs: &[(&str, u32, u32)]) -> Self {
        let mut fc = Self::new();
        for (name, count, level) in pairs {
            fc.set(name, *count, *level);
        }
        fc
    }

    /// Returns a `FacilityCounts` where every facility (known or not) reports level 99,
    /// count 1 — used to list all possible items regardless of facility-level gating.
    pub fn show_all_levels() -> Self {
        Self {
            facilities: std::collections::HashMap::new(),
            default_level: 99,
        }
    }

    /// Sets the count and level for a facility by name.
    pub fn set(&mut self, facility: &str, count: u32, level: u32) -> &mut Self {
        self.facilities.insert(facility.to_string(), (count, level));
        self
    }

    /// Returns the count for a given facility name.
    ///
    /// # Arguments
    ///
    /// * `facility` - The name of the facility (e.g., "Farmland", "Carousel Mill")
    ///
    /// # Returns
    ///
    /// The number of that facility type available. Returns 1 for unset/unknown facility types.
    pub fn get_count(&self, facility: &str) -> u32 {
        self.facilities.get(facility).map(|(c, _)| *c).unwrap_or(1)
    }

    /// Returns the level for a given facility name.
    ///
    /// # Arguments
    ///
    /// * `facility` - The name of the facility (e.g., "Farmland", "Carousel Mill")
    ///
    /// # Returns
    ///
    /// The level of that facility type. Returns 1 for unset/unknown facility types.
    pub fn get_level(&self, facility: &str) -> u32 {
        self.facilities
            .get(facility)
            .map(|(_, l)| *l)
            .unwrap_or(self.default_level)
    }

    /// Checks if a facility can produce an item at the given required level.
    ///
    /// # Arguments
    ///
    /// * `facility` - The name of the facility
    /// * `required_level` - The level required by the item
    ///
    /// # Returns
    ///
    /// `true` if the facility level is >= required level
    pub fn can_produce(&self, facility: &str, required_level: u32) -> bool {
        self.get_level(facility) >= required_level
    }
}

/// Tracks the levels of item upgrade modules.
///
/// Modules unlock upgraded versions of items with better yields or sell values.
///
/// # Example
///
/// ```
/// use aniimax::models::ModuleLevels;
///
/// let modules = ModuleLevels {
///     ecological_module: 2,  // Unlocks high-speed wheat and willow
///     kitchen_module: 2,     // Unlocks super wheat flour
///     mineral_detector: 1,   // Unlocks high-speed rock
///     crafting_module: 1,    // Unlocks advanced wood carving
/// };
///
/// assert!(modules.can_use("ecological_module", 1));
/// ```
#[derive(Debug, Clone)]
pub struct ModuleLevels {
    /// Level of Ecological Module (unlocks high-speed wheat at 1, high-speed willow at 2)
    pub ecological_module: u32,
    /// Level of Kitchen Module (unlocks super wheat flour at 2)
    pub kitchen_module: u32,
    /// Level of Mineral Detector (unlocks high-speed rock at 1)
    pub mineral_detector: u32,
    /// Level of Crafting Module (unlocks advanced wood carving at 1)
    pub crafting_module: u32,
}

impl Default for ModuleLevels {
    fn default() -> Self {
        ModuleLevels {
            ecological_module: 0,
            kitchen_module: 0,
            mineral_detector: 0,
            crafting_module: 0,
        }
    }
}

impl ModuleLevels {
    /// Checks if a module meets the required level.
    ///
    /// # Arguments
    ///
    /// * `module_name` - The name of the module
    /// * `required_level` - The level required
    ///
    /// # Returns
    ///
    /// `true` if the module level is >= required level
    pub fn can_use(&self, module_name: &str, required_level: u32) -> bool {
        self.get_level(module_name) >= required_level
    }

    /// Returns the level for a given module name.
    pub fn get_level(&self, module_name: &str) -> u32 {
        match module_name {
            "ecological_module" => self.ecological_module,
            "kitchen_module" => self.kitchen_module,
            "mineral_detector" => self.mineral_detector,
            "crafting_module" => self.crafting_module,
            _ => 0,
        }
    }
}

// ============================================================================
// CSV Row Structures
// ============================================================================

/// CSV row structure for Farmland items.
#[derive(Debug, Deserialize)]
pub struct FarmlandRow {
    /// Item name
    pub name: String,
    /// Cost to plant
    pub cost: f64,
    /// Sell value per unit
    pub sell_value: f64,
    /// Production time in seconds
    pub production_time: f64,
    /// Number of items yielded
    #[serde(rename = "yield")]
    pub yield_amount: u32,
    /// Energy consumed (optional)
    pub energy: Option<f64>,
    /// Required facility level
    pub facility_level: u32,
    /// Module requirement (format: "module_name:level" or empty)
    #[serde(default)]
    pub module_requirement: Option<String>,
}

/// CSV row structure for Woodland items.
#[derive(Debug, Deserialize)]
pub struct WoodlandRow {
    /// Item name
    pub name: String,
    /// Cost to plant
    pub cost: f64,
    /// Currency type when selling
    pub sell_currency: String,
    /// Sell value per unit
    pub sell_value: f64,
    /// Production time in seconds
    pub production_time: f64,
    /// Number of items yielded
    #[serde(rename = "yield")]
    pub yield_amount: u32,
    /// Secondary Wood Blocks yield (new-beta byproduct, used for Homeland upgrades)
    #[serde(default)]
    pub byproduct_yield: Option<u32>,
    /// Energy consumed (may be "NULL" string)
    pub energy: Option<String>,
    /// Required facility level
    pub facility_level: u32,
    /// Module requirement (format: "module_name:level" or empty)
    #[serde(default)]
    pub module_requirement: Option<String>,
}

/// CSV row structure for Mineral Pile items.
///
/// New-beta Mineral Pile items are workload-based (Aniimo-dispatch driven) rather than
/// flat-time, so this row carries `workload` instead of `production_time`. See
/// [`WORKLOAD_RATE_ESTIMATE`] for how workload is converted into an estimated time.
#[derive(Debug, Deserialize)]
pub struct MineralRow {
    /// Item name
    pub name: String,
    /// Currency type when selling
    pub sell_currency: String,
    /// Sell value per unit
    pub sell_value: f64,
    /// Workload stat — converted to an estimated production time via
    /// [`WORKLOAD_RATE_ESTIMATE`]
    pub workload: f64,
    /// Number of items yielded
    #[serde(rename = "yield")]
    pub yield_amount: u32,
    /// Secondary Mineral Sand yield (new-beta byproduct, used for Homeland upgrades)
    #[serde(default)]
    pub byproduct_yield: Option<u32>,
    /// Required facility level
    pub facility_level: u32,
    /// Module requirement (format: "module_name:level" or empty)
    #[serde(default)]
    pub module_requirement: Option<String>,
}

/// CSV row structure for processing facilities with energy tracking.
///
/// Carries either a flat `production_time` (old-style facilities not yet updated for the new
/// beta) or a `workload` (new-beta facilities — converted to time via
/// [`WORKLOAD_RATE_ESTIMATE`]). At least one of the two must be present in the CSV.
/// `sell_currency` is optional (defaults to "coins" if the column is absent) — added because
/// new-beta processing facilities (e.g. Claw Game Cooker) can sell for coins or Bud Tickets.
#[derive(Debug, Deserialize)]
pub struct ProcessingRowWithEnergy {
    /// Item name
    pub name: String,
    /// Required raw material name(s), semicolon-separated if multiple
    pub raw_materials: String,
    /// Amount of raw materials needed, semicolon-separated if multiple
    pub required_amount: String,
    /// Sell value per unit
    pub sell_value: f64,
    /// Currency the item sells for ("coins" or "bud_tickets"). Defaults to "coins" if absent.
    #[serde(default)]
    pub sell_currency: Option<String>,
    /// Production time in seconds (old-style flat-time facilities)
    #[serde(default)]
    pub production_time: Option<f64>,
    /// Workload stat (new-beta facilities) — converted to time via [`WORKLOAD_RATE_ESTIMATE`]
    #[serde(default)]
    pub workload: Option<f64>,
    /// Energy consumed (optional for items that don't consume energy)
    #[serde(default, deserialize_with = "crate::deserialize_optional_f64")]
    pub energy: Option<f64>,
    /// Required facility level
    pub facility_level: u32,
    /// Module requirement (format: "module_name:level" or empty)
    #[serde(default)]
    pub module_requirement: Option<String>,
}

/// CSV row structure for processing facilities without energy tracking.
///
/// Carries either a flat `production_time` (old-style facilities not yet updated for the new
/// beta) or a `workload` (new-beta facilities — converted to time via
/// [`WORKLOAD_RATE_ESTIMATE`]). At least one of the two must be present in the CSV.
/// `sell_currency` is optional (defaults to "coins" if the column is absent) — added because
/// new-beta Crafting Table items can sell for coins or Bud Tickets depending on the recipe.
#[derive(Debug, Deserialize)]
pub struct ProcessingRowNoEnergy {
    /// Item name
    pub name: String,
    /// Required raw material name(s), semicolon-separated if multiple
    pub raw_materials: String,
    /// Amount of raw materials needed, semicolon-separated if multiple
    pub required_amount: String,
    /// Sell value per unit
    pub sell_value: f64,
    /// Currency the item sells for ("coins" or "bud_tickets"). Defaults to "coins" if absent.
    #[serde(default)]
    pub sell_currency: Option<String>,
    /// Production time in seconds (old-style flat-time facilities)
    #[serde(default)]
    pub production_time: Option<f64>,
    /// Workload stat (new-beta facilities) — converted to time via [`WORKLOAD_RATE_ESTIMATE`]
    #[serde(default)]
    pub workload: Option<f64>,
    /// Required facility level
    pub facility_level: u32,
    /// Module requirement (format: "module_name:level" or empty)
    #[serde(default)]
    pub module_requirement: Option<String>,
}

/// CSV row structure for Nimbus Bed items.
///
/// New-beta Nimbus Bed items are workload-based (Aniimo-dispatch driven, requiring a specific
/// Aniimo Family) rather than flat-time. See [`WORKLOAD_RATE_ESTIMATE`] for the time
/// conversion. (Note: a Nimbus-Bed-specific rate was originally derived from the Petals data
/// point at 140% efficiency — 2250/723.8 ≈ 3.1086 — but was superseded by the universal
/// [`WORKLOAD_RATE_ESTIMATE`] once workload tier values started appearing identically across
/// unrelated facilities; see that constant's docs.)
#[derive(Debug, Deserialize)]
pub struct NimbusBedRow {
    /// Item name
    pub name: String,
    /// Sell value per unit
    pub sell_value: f64,
    /// Workload stat — converted to an estimated production time via
    /// [`WORKLOAD_RATE_ESTIMATE`]
    pub workload: f64,
    /// Number of items yielded
    #[serde(rename = "yield")]
    pub yield_amount: u32,
}
