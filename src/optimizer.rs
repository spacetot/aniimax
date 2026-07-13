//! Production optimization algorithms for the Aniimo optimizer.
//!
//! This module contains the core optimization logic that calculates
//! production efficiencies and finds the best production paths to
//! achieve currency goals.

use std::collections::{HashMap, HashSet};

use microlp::{ComparisonOp, OptimizationDirection, Problem};

use crate::models::{
    EnergyItemEfficiency, GoalResult, PlanProduct, PlanStep, PlanStepStatus, ProductionPlan,
    FacilityCounts, ModuleLevels, ProductionEfficiency, ProductionItem, ProductionPath,
    ProductionStep,
};

/// Calculates the optimal allocation of facilities to minimize production time
/// when producing multiple different materials.
/// 
/// Given:
/// - `materials`: Vec of (material_name, batches_needed, time_per_batch)
/// - `total_facilities`: Total number of facilities available
/// 
/// Returns: Vec of (material_name, batches_needed, optimal_facilities_to_allocate)
/// 
/// Uses binary search on the answer for O(M * sqrt(B) * log(M * sqrt(B))) complexity,
/// where M = number of materials, B = max batches.
fn calculate_optimal_facility_allocation(
    materials: &[(String, u32, f64)],
    total_facilities: u32,
) -> Vec<(String, u32, u32)> {
    if materials.is_empty() {
        return vec![];
    }
    
    if total_facilities == 0 {
        return materials.iter()
            .map(|(name, batches, _)| (name.clone(), *batches, 0))
            .collect();
    }
    
    if materials.len() == 1 {
        // Single material gets all facilities
        return vec![(materials[0].0.clone(), materials[0].1, total_facilities)];
    }
    
    // Filter to only materials that need production (batches > 0 and time > 0)
    let active_materials: Vec<(usize, u32, f64)> = materials.iter()
        .enumerate()
        .filter(|(_, (_, batches, time))| *batches > 0 && *time > 0.0)
        .map(|(i, (_, batches, time))| (i, *batches, *time))
        .collect();
    
    if active_materials.is_empty() {
        // No active materials - just return with 0 allocations
        return materials.iter()
            .map(|(name, batches, _)| (name.clone(), *batches, 0))
            .collect();
    }
    
    // Check if we have enough facilities (at least 1 per active material)
    if (active_materials.len() as u32) > total_facilities {
        // Not enough - distribute proportionally
        return distribute_proportionally(materials, total_facilities);
    }
    
    // Collect all possible completion times using sqrt decomposition
    // For ceil(b/k) where k goes from 1 to b, there are only O(sqrt(b)) distinct values
    let mut candidate_times: Vec<f64> = Vec::new();
    for (_, batches, time) in &active_materials {
        if *batches == 0 || *time <= 0.0 {
            continue;
        }
        let mut k = 1u32;
        while k <= *batches {
            let rounds = (*batches + k - 1) / k; // ceil(batches / k)
            candidate_times.push(rounds as f64 * time);
            // Jump to next k that gives a different ceil value
            if rounds > 1 {
                k = *batches / (rounds - 1);
                if k * (rounds - 1) < *batches {
                    k += 1;
                }
            } else {
                break;
            }
        }
        // Also add the case where we use all facilities for this material
        if total_facilities > 0 {
            candidate_times.push(((*batches + total_facilities - 1) / total_facilities) as f64 * time);
        }
    }
    
    // Handle empty candidates
    if candidate_times.is_empty() {
        return materials.iter()
            .enumerate()
            .map(|(i, (name, batches, _))| {
                let alloc = if i < active_materials.len() { 1 } else { 0 };
                (name.clone(), *batches, alloc)
            })
            .collect();
    }
    
    // Sort and deduplicate
    candidate_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    candidate_times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    
    // Binary search for minimum feasible time
    let optimal_time = binary_search_min_time(&candidate_times, &active_materials, total_facilities);
    
    // Calculate the allocation for this optimal time
    calculate_allocation_for_time(materials, &active_materials, optimal_time, total_facilities)
}

/// Binary search to find the minimum feasible completion time
fn binary_search_min_time(
    candidate_times: &[f64],
    active_materials: &[(usize, u32, f64)],
    total_facilities: u32,
) -> f64 {
    if candidate_times.is_empty() {
        return 0.0;
    }
    
    let mut lo = 0;
    let mut hi = candidate_times.len();
    
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if is_time_feasible(candidate_times[mid], active_materials, total_facilities) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    
    if lo < candidate_times.len() {
        candidate_times[lo]
    } else {
        // Fallback - use the largest candidate time
        candidate_times.last().copied().unwrap_or(0.0)
    }
}

/// Check if a target completion time is achievable with the given facilities
fn is_time_feasible(
    target_time: f64,
    active_materials: &[(usize, u32, f64)],
    total_facilities: u32,
) -> bool {
    let mut facilities_needed = 0u32;
    
    for (_, batches, time) in active_materials {
        if *time <= 0.0 {
            continue;
        }
        
        let max_rounds = (target_time / time).floor() as u32;
        if max_rounds == 0 {
            return false; // Can't complete even one round in time
        }
        
        // Minimum facilities needed: ceil(batches / max_rounds)
        let min_facilities = (*batches + max_rounds - 1) / max_rounds;
        facilities_needed = facilities_needed.saturating_add(min_facilities);
        
        if facilities_needed > total_facilities {
            return false;
        }
    }
    
    true
}

/// Calculate the actual facility allocation for a given target time
fn calculate_allocation_for_time(
    materials: &[(String, u32, f64)],
    active_materials: &[(usize, u32, f64)],
    target_time: f64,
    total_facilities: u32,
) -> Vec<(String, u32, u32)> {
    let mut result: Vec<(String, u32, u32)> = materials.iter()
        .map(|(name, batches, _)| (name.clone(), *batches, 0))
        .collect();
    
    // Calculate minimum facilities needed for each active material
    let mut allocations: Vec<(usize, u32)> = Vec::new();
    let mut total_min = 0u32;
    
    for (idx, batches, time) in active_materials {
        if *time <= 0.0 {
            allocations.push((*idx, 1));
            total_min += 1;
            continue;
        }
        
        let max_rounds = (target_time / time).floor() as u32;
        let min_facilities = if max_rounds == 0 {
            *batches // Need one facility per batch (shouldn't happen if time is feasible)
        } else {
            (*batches + max_rounds - 1) / max_rounds
        };
        
        allocations.push((*idx, min_facilities));
        total_min += min_facilities;
    }
    
    // Distribute remaining facilities to reduce time further where possible
    let mut remaining = total_facilities.saturating_sub(total_min);
    
    // Assign minimum allocations first
    for (idx, min_fac) in &allocations {
        result[*idx].2 = *min_fac;
    }
    
    // Distribute remaining facilities greedily - give to the material that benefits most
    while remaining > 0 {
        let mut best_improvement = 0.0f64;
        let mut best_idx = None;
        
        for (idx, batches, time) in active_materials {
            let current_facilities = result[*idx].2;
            if current_facilities == 0 {
                continue;
            }
            
            let current_rounds = (*batches + current_facilities - 1) / current_facilities;
            let new_rounds = (*batches + current_facilities) / (current_facilities + 1);
            
            if new_rounds < current_rounds {
                let improvement = (current_rounds - new_rounds) as f64 * time;
                if improvement > best_improvement {
                    best_improvement = improvement;
                    best_idx = Some(*idx);
                }
            }
        }
        
        if let Some(idx) = best_idx {
            result[idx].2 += 1;
            remaining -= 1;
        } else {
            // No improvement possible, distribute to first active material
            if let Some((idx, _, _)) = active_materials.first() {
                result[*idx].2 += remaining;
            }
            break;
        }
    }
    
    result
}

/// Distribute facilities proportionally when there aren't enough
fn distribute_proportionally(
    materials: &[(String, u32, f64)],
    total_facilities: u32,
) -> Vec<(String, u32, u32)> {
    let total_batches: u32 = materials.iter().map(|(_, b, _)| b).sum();
    
    if total_batches == 0 {
        return materials.iter()
            .map(|(name, batches, _)| (name.clone(), *batches, 0))
            .collect();
    }
    
    let mut result: Vec<(String, u32, u32)> = Vec::with_capacity(materials.len());
    let mut remaining = total_facilities;
    
    for (i, (name, batches, _)) in materials.iter().enumerate() {
        let alloc = if i == materials.len() - 1 {
            remaining
        } else if *batches > 0 {
            let frac = (*batches as f64 / total_batches as f64 * total_facilities as f64).round() as u32;
            frac.min(remaining).max(1)
        } else {
            0
        };
        result.push((name.clone(), *batches, alloc));
        remaining = remaining.saturating_sub(alloc);
    }
    
    result
}

/// Result of calculating production requirements for an item
#[derive(Debug, Clone)]
struct ProductionRequirements {
    /// Total time to produce the item (including all dependencies)
    total_time: f64,
    /// Total energy consumed (including all dependencies)
    total_energy: Option<f64>,
    /// Total cost of raw materials
    total_cost: f64,
    /// Names of all raw materials in the chain
    raw_names: Vec<String>,
    /// Primary facility for the base raw material
    primary_facility: Option<String>,
    /// All facilities used in this production chain (for conflict detection)
    all_facilities: HashSet<String>,
    /// Intermediate processing steps: (item_name, facility, amount_per_parent_batch)
    intermediate_steps: Vec<(String, String, u32)>,
    /// Whether this production chain is valid
    is_valid: bool,
}

/// Resolves a raw material name to the actual item that would supply it, preferring the quick_
/// variant when it exists and is usable — same substitution rule used everywhere else raw
/// material names are resolved. Keys the item map by owned `String` since that's what
/// `calculate_efficiencies` and the resource-demand functions below use.
fn resolve_raw_material<'a>(
    name: &str,
    item_map: &HashMap<String, &'a ProductionItem>,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
) -> Option<&'a ProductionItem> {
    let high_speed_name = format!("quick_{}", name);
    if let Some(&hs) = item_map.get(&high_speed_name) {
        let can_use = hs
            .module_requirement
            .as_ref()
            .map_or(true, |(m, l)| module_levels.can_use(m, *l));
        let can_produce = facility_counts.can_produce(&hs.facility, hs.facility_level);
        if can_use && can_produce {
            return Some(hs);
        }
    }
    item_map.get(name).copied()
}

/// Walks `item`'s full ingredient tree, accumulating — per FACILITY touched anywhere in the
/// tree, including `item`'s own facility — total *utilization*: batches/sec of whatever runs
/// there, weighted by that item's own `production_time`, required per one batch/sec of the
/// tree's root. Utilization (a dimensionless "fraction of one facility's continuous operation")
/// is what makes sharing correct in every shape it comes in, because it's always additive:
///
/// - The same item needed via two different branches — soy_sauce_tofu needs both soy_sauce
///   (Bouncy Brew Keg) and tofu (Carousel Mill), and both independently need soybean from the
///   same Farmland. Computing each branch's rate in isolation (the old approach) let each assume
///   it alone could draw all 20 Farmland's worth of soybean, silently doubling the effective
///   rate. Here both branches' soybean utilization lands in the same `Farmland` entry and sums.
/// - Two DIFFERENT items hosted at the same facility for the same chain — e.g. Claw Game Cooker
///   both turning sugarcane into rock_candy AND assembling rock_candy+tofu into tofu_cake, or
///   Woodland growing both coconut and lemon for a soap recipe. These aren't the same item, but
///   they're still time-sharing one facility's capacity, so their utilization sums too.
///
/// Keying by facility (not item name) is what makes both cases fall out of one formula instead
/// of needing separate handling.
fn accumulate_demand<'a>(
    item: &'a ProductionItem,
    ratio: f64,
    item_map: &HashMap<String, &'a ProductionItem>,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
    demand: &mut HashMap<&'a str, (f64, Vec<&'a ProductionItem>)>,
    depth: u32,
) {
    if depth > 8 {
        return; // guard against unexpected circular references
    }
    let entry = demand.entry(item.facility.as_str()).or_insert((0.0, Vec::new()));
    entry.0 += ratio * item.production_time;
    if !entry.1.iter().any(|hosted| hosted.name == item.name) {
        entry.1.push(item);
    }
    let Some(ref raw_mats) = item.raw_materials else {
        return;
    };
    let required_amounts = item.required_amount.as_deref().unwrap_or(&[]);
    for (i, raw_mat) in raw_mats.iter().enumerate() {
        let Some(resolved) = resolve_raw_material(raw_mat, item_map, facility_counts, module_levels)
        else {
            continue;
        };
        let required_per_batch = required_amounts.get(i).copied().unwrap_or(1) as f64;
        if required_per_batch <= 0.0 {
            continue;
        }
        let sub_ratio = ratio * required_per_batch / resolved.yield_amount as f64;
        accumulate_demand(
            resolved,
            sub_ratio,
            item_map,
            facility_counts,
            module_levels,
            demand,
            depth + 1,
        );
    }
}

/// Convenience wrapper around [`accumulate_demand`] that returns the completed demand map for
/// `item`'s whole ingredient tree, rooted at `item` itself (ratio 1.0).
fn compute_resource_demand<'a>(
    item: &'a ProductionItem,
    item_map: &HashMap<String, &'a ProductionItem>,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
) -> HashMap<&'a str, (f64, Vec<&'a ProductionItem>)> {
    let mut demand = HashMap::new();
    accumulate_demand(item, 1.0, item_map, facility_counts, module_levels, &mut demand, 0);
    demand
}

/// The true bottleneck-limited batches/sec achievable at the root of a resource-demand map: the
/// minimum, over every facility touched anywhere in the tree, of that facility's own batch
/// capacity divided by its accumulated utilization (see [`accumulate_demand`]) — a facility
/// touched by only one item reduces to the familiar `facility_count / production_time /
/// required_per_batch`; a facility touched by multiple items (whether the same item via
/// different branches, or different items time-sharing it) gets their utilization summed
/// automatically, since they share one entry in the map.
fn batch_rate_bound(
    demand: &HashMap<&str, (f64, Vec<&ProductionItem>)>,
    facility_counts: &FacilityCounts,
) -> f64 {
    demand
        .iter()
        .map(|(facility, (utilization, _))| {
            if *utilization <= 0.0 {
                return f64::INFINITY;
            }
            let count = facility_counts.get_count(facility) as f64;
            if count <= 0.0 {
                0.0
            } else {
                count / utilization
            }
        })
        .fold(f64::INFINITY, f64::min)
}

/// Recursively calculates production requirements for an item.
///
/// This handles both simple raw materials and processed items that may
/// require other processed items as ingredients (e.g., caramel_nut_chips requires nuts).
fn calculate_item_requirements(
    item_name: &str,
    required_amount: f64,
    item_map: &HashMap<String, &ProductionItem>,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
    fertilizer_time_per_unit: f64,
    nimbus_bed_count: f64,
    visited: &mut HashSet<String>, // Prevent infinite recursion
) -> ProductionRequirements {
    // Check for circular dependencies
    if visited.contains(item_name) {
        return ProductionRequirements {
            total_time: 0.0,
            total_energy: None,
            total_cost: 0.0,
            raw_names: vec![],
            primary_facility: None,
            all_facilities: HashSet::new(),
            intermediate_steps: vec![],
            is_valid: false,
        };
    }
    
    // Try to find the best variant of this item (check quick_ variant first — new-beta name
    // for what used to be the high_speed_ variant)
    let high_speed_name = format!("quick_{}", item_name);
    let (item, actual_name) = {
        // Check if quick_ variant exists and is usable
        if let Some(hs_item) = item_map.get(&high_speed_name) {
            // Verify we can use the quick_ variant (check module requirements)
            let can_use_hs = if let Some((ref module_name, required_level)) = hs_item.module_requirement {
                module_levels.can_use(module_name, required_level)
            } else {
                true
            };
            // Also check facility requirements
            let can_produce_hs = facility_counts.can_produce(&hs_item.facility, hs_item.facility_level);

            if can_use_hs && can_produce_hs {
                // Use quick_ variant - it produces more in same/less time
                (*hs_item, high_speed_name.as_str())
            } else if let Some(base_item) = item_map.get(item_name) {
                // Fall back to base variant
                (*base_item, item_name)
            } else {
                return ProductionRequirements {
                    total_time: 0.0,
                    total_energy: None,
                    total_cost: 0.0,
                    raw_names: vec![],
                    primary_facility: None,
                    all_facilities: HashSet::new(),
                    intermediate_steps: vec![],
                    is_valid: false,
                };
            }
        } else if let Some(base_item) = item_map.get(item_name) {
            // No high_speed variant, use base
            (*base_item, item_name)
        } else {
            return ProductionRequirements {
                total_time: 0.0,
                total_energy: None,
                total_cost: 0.0,
                raw_names: vec![],
                primary_facility: None,
                all_facilities: HashSet::new(),
                intermediate_steps: vec![],
                is_valid: false,
            };
        }
    };
    
    // Check if facility can produce this item
    if !facility_counts.can_produce(&item.facility, item.facility_level) {
        return ProductionRequirements {
            total_time: 0.0,
            total_energy: None,
            total_cost: 0.0,
            raw_names: vec![],
            primary_facility: None,
            all_facilities: HashSet::new(),
            intermediate_steps: vec![],
            is_valid: false,
        };
    }
    
    // Check module requirements
    if let Some((ref module_name, required_level)) = item.module_requirement {
        if !module_levels.can_use(module_name, required_level) {
            return ProductionRequirements {
                total_time: 0.0,
                total_energy: None,
                total_cost: 0.0,
                raw_names: vec![],
                primary_facility: None,
                all_facilities: HashSet::new(),
                intermediate_steps: vec![],
                is_valid: false,
            };
        }
    }
    
    // Check fertilizer requirements
    if item.requires_fertilizer && nimbus_bed_count == 0.0 {
        return ProductionRequirements {
            total_time: 0.0,
            total_energy: None,
            total_cost: 0.0,
            raw_names: vec![],
            primary_facility: None,
            all_facilities: HashSet::new(),
            intermediate_steps: vec![],
            is_valid: false,
        };
    }
    
    visited.insert(actual_name.to_string());
    
    let result = if let Some(ref raw_mats) = item.raw_materials {
        // This is a processed item - recursively calculate requirements for each ingredient
        let required_amounts = item.required_amount.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        
        let mut max_ingredient_time = 0.0;
        let mut total_ingredient_energy: Option<f64> = None;
        let mut total_ingredient_cost = 0.0;
        let mut all_raw_names: Vec<String> = Vec::new();
        let mut primary_facility: Option<String> = None;
        let mut all_facilities: HashSet<String> = HashSet::new();
        let mut intermediate_steps: Vec<(String, String, u32)> = Vec::new();
        
        // Add THIS item's processing facility
        all_facilities.insert(item.facility.clone());
        
        // Calculate how many batches of this processed item we need
        let batches_needed = (required_amount / item.yield_amount as f64).ceil();
        
        for (i, raw_mat) in raw_mats.iter().enumerate() {
            let ingredient_required_per_batch = required_amounts.get(i).copied().unwrap_or(1);
            let ingredient_required = ingredient_required_per_batch as f64 * batches_needed;
            
            let ingredient_reqs = calculate_item_requirements(
                raw_mat,
                ingredient_required,
                item_map,
                facility_counts,
                module_levels,
                fertilizer_time_per_unit,
                nimbus_bed_count,
                visited,
            );
            
            if !ingredient_reqs.is_valid {
                visited.remove(actual_name);
                return ProductionRequirements {
                    total_time: 0.0,
                    total_energy: None,
                    total_cost: 0.0,
                    raw_names: vec![],
                    primary_facility: None,
                    all_facilities: HashSet::new(),
                    intermediate_steps: vec![],
                    is_valid: false,
                };
            }
            
            // Ingredients can be gathered in parallel, so take max time
            if ingredient_reqs.total_time > max_ingredient_time {
                max_ingredient_time = ingredient_reqs.total_time;
            }
            
            // Energy and cost are additive
            if let Some(e) = ingredient_reqs.total_energy {
                total_ingredient_energy = Some(total_ingredient_energy.unwrap_or(0.0) + e);
            }
            total_ingredient_cost += ingredient_reqs.total_cost;
            
            all_raw_names.extend(ingredient_reqs.raw_names);
            if primary_facility.is_none() {
                primary_facility = ingredient_reqs.primary_facility;
            }
            
            // Merge all facilities from ingredients
            all_facilities.extend(ingredient_reqs.all_facilities);
            
            // Propagate intermediate steps from ingredients
            intermediate_steps.extend(ingredient_reqs.intermediate_steps);
            
            // If this ingredient is itself a processed item, add it as an intermediate step
            // Check by looking up the item and seeing if it has raw_materials
            let high_speed_mat = format!("quick_{}", raw_mat);
            let mat_item = item_map.get(&high_speed_mat)
                .filter(|hs| {
                    let can_use = if let Some((ref m, l)) = hs.module_requirement {
                        module_levels.can_use(m, l)
                    } else { true };
                    can_use && facility_counts.can_produce(&hs.facility, hs.facility_level)
                })
                .or_else(|| item_map.get(raw_mat.as_str()));
            
            if let Some(mat) = mat_item {
                if mat.raw_materials.is_some() {
                    // This is a processed intermediate - add it as a step
                    intermediate_steps.push((
                        mat.name.clone(),
                        mat.facility.clone(),
                        ingredient_required_per_batch,
                    ));
                }
            }
        }
        
        // Add processing time for this item
        let processing_facility_count = facility_counts.get_count(&item.facility) as f64;
        let processing_time = item.production_time * (batches_needed / processing_facility_count).ceil();
        
        // Add processing energy
        let total_energy = match (total_ingredient_energy, item.energy) {
            (Some(ie), Some(pe)) => Some(ie + pe * batches_needed),
            (Some(ie), None) => Some(ie),
            (None, Some(pe)) => Some(pe * batches_needed),
            (None, None) => None,
        };
        
        ProductionRequirements {
            total_time: max_ingredient_time + processing_time,
            total_energy,
            total_cost: total_ingredient_cost,
            raw_names: all_raw_names,
            primary_facility,
            all_facilities,
            intermediate_steps,
            is_valid: true,
        }
    } else {
        // This is a base raw material
        let facility_count = facility_counts.get_count(&item.facility) as f64;
        let batches_needed = (required_amount / item.yield_amount as f64).ceil();
        
        // Calculate time with parallel facilities
        let time_per_batch = item.production_time;
        let parallel_batches = (batches_needed / facility_count).ceil();
        
        // Add fertilizer time if required
        let fertilizer_time = if item.requires_fertilizer {
            fertilizer_time_per_unit * batches_needed
        } else {
            0.0
        };
        
        let total_time = time_per_batch * parallel_batches + fertilizer_time;
        let total_energy = item.energy.map(|e| e * batches_needed);
        let total_cost = item.cost.unwrap_or(0.0) * batches_needed;
        
        let mut all_facilities = HashSet::new();
        all_facilities.insert(item.facility.clone());
        
        ProductionRequirements {
            total_time,
            total_energy,
            total_cost,
            raw_names: vec![actual_name.to_string()],
            primary_facility: Some(item.facility.clone()),
            all_facilities,
            intermediate_steps: vec![],
            is_valid: true,
        }
    };
    
    visited.remove(actual_name);
    result
}

/// Calculates efficiency metrics for all production items.
///
/// This function evaluates each production item based on:
/// - Profit per second (time efficiency)
/// - Profit per energy unit (energy efficiency)
/// - Total production time including raw material gathering
/// - Parallel production capability based on facility counts
///
/// # Arguments
///
/// * `items` - All available production items
/// * `target_currency` - The currency to optimize for ("coins" or "bud_tickets")
/// * `facility_counts` - Configuration for each facility (count and level)
/// * `module_levels` - Configuration for each item upgrade module level
///
/// # Returns
///
/// A vector of [`ProductionEfficiency`] structs for all valid production options.
///
/// # Filtering
///
/// Items are filtered out if:
/// - Their facility level exceeds the specific facility's level
/// - They require a module level that isn't met
/// - They don't produce the target currency
/// - Their required raw materials aren't available at the raw material facility's level
///
/// # Example
///
/// ```no_run
/// use aniimax::optimizer::calculate_efficiencies;
/// use aniimax::models::{FacilityCounts, ModuleLevels};
/// use aniimax::data::load_all_data;
/// use std::path::Path;
///
/// let items = load_all_data(Path::new("data")).unwrap();
/// let counts = FacilityCounts::from_pairs(&[
///     ("Farmland", 4, 3),        // 4 farmlands at level 3
///     ("Woodland", 1, 2),        // 1 woodland at level 2
///     ("Mineral Pile", 1, 1),    // 1 mineral pile at level 1
///     ("Carousel Mill", 2, 2),   // 2 carousel mills at level 2
///     ("Jukebox Dryer", 1, 1),
///     ("Crafting Table", 1, 1),
///     ("Nimbus Bed", 1, 1),      // 1 nimbus bed (Wool/Petals)
/// ]);
/// let modules = ModuleLevels::default();
///
/// let efficiencies = calculate_efficiencies(&items, "coins", &counts, &modules);
/// ```
pub fn calculate_efficiencies(
    items: &[ProductionItem],
    target_currency: &str,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
) -> Vec<ProductionEfficiency> {
    let item_map: HashMap<String, &ProductionItem> =
        items.iter().map(|i| (i.name.clone(), i)).collect();

    // Find fertilizer item for calculating fertilizer production time
    let fertilizer_item = item_map.get("fertilizer");
    let nimbus_bed_count = facility_counts.get_count("Nimbus Bed") as f64;

    // Calculate time to produce one fertilizer (if Nimbus Bed is available)
    // Fertilizer: 30 yield per 1800s, so each fertilizer takes 60s to produce
    let fertilizer_time_per_unit = fertilizer_item
        .map(|f| f.production_time / (f.yield_amount as f64 * nimbus_bed_count.max(1.0)))
        .unwrap_or(0.0);

    let mut efficiencies = Vec::new();

    for item in items {
        // Filter by facility level (check if this facility can produce this item)
        if !facility_counts.can_produce(&item.facility, item.facility_level) {
            continue;
        }

        // Filter by module requirement
        if let Some((ref module_name, required_level)) = item.module_requirement {
            if !module_levels.can_use(module_name, required_level) {
                continue;
            }
        }

        // Filter by target currency
        if item.sell_currency != target_currency {
            continue;
        }

        // Filter out items that require fertilizer if no Nimbus Bed is available
        if item.requires_fertilizer && nimbus_bed_count == 0.0 {
            continue;
        }

        let (total_time, steady_state_time, total_energy, raw_cost, requires_raw, raw_facility, all_facilities, intermediate_steps, raw_material_details, fertilizer_per_batch, facility_demand) =
            if let Some(ref raw_mats) = item.raw_materials {
                // This is a processed item - use recursive calculation to handle nested dependencies
                let required_amounts = item.required_amount.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
                
                // Track totals across all raw materials
                let mut max_ingredient_time = 0.0;
                let mut total_ingredient_energy: Option<f64> = None;
                let mut total_ingredient_cost = 0.0;
                let mut all_raw_names: Vec<String> = Vec::new();
                let mut primary_facility: Option<String> = None;
                let mut all_facilities_collected: HashSet<String> = HashSet::new();
                let mut all_intermediate_steps: Vec<(String, String, u32)> = Vec::new();
                // (name, amount_per_batch, time_per_batch, facility) - includes facility for filtering
                let mut raw_material_details_collected: Vec<(String, u32, f64, String)> = Vec::new();
                let mut skip_item = false;
                // Track total fertilizer needed per processed batch
                let mut fertilizer_per_batch: u32 = 0;
                
                // Add THIS item's processing facility
                all_facilities_collected.insert(item.facility.clone());

                for (i, raw_mat) in raw_mats.iter().enumerate() {
                    let required = required_amounts.get(i).copied().unwrap_or(1);
                    
                    let mut visited = HashSet::new();
                    let reqs = calculate_item_requirements(
                        raw_mat,
                        required as f64,
                        &item_map,
                        facility_counts,
                        module_levels,
                        fertilizer_time_per_unit,
                        nimbus_bed_count,
                        &mut visited,
                    );
                    
                    if !reqs.is_valid {
                        skip_item = true;
                        break;
                    }
                    
                    // Ingredients can be gathered in parallel, so take max time
                    if reqs.total_time > max_ingredient_time {
                        max_ingredient_time = reqs.total_time;
                    }
                    
                    // Energy and cost are additive
                    if let Some(e) = reqs.total_energy {
                        total_ingredient_energy = Some(total_ingredient_energy.unwrap_or(0.0) + e);
                    }
                    total_ingredient_cost += reqs.total_cost;
                    
                    all_raw_names.extend(reqs.raw_names);
                    if primary_facility.is_none() {
                        primary_facility = reqs.primary_facility;
                    }
                    
                    // Merge all facilities from ingredients
                    all_facilities_collected.extend(reqs.all_facilities);
                    
                    // Collect intermediate steps from recursive requirements
                    all_intermediate_steps.extend(reqs.intermediate_steps);
                    
                    // If this raw_mat is itself a processed item, add it as an intermediate step
                    let high_speed_name = format!("quick_{}", raw_mat);
                    let mat_item = item_map.get(&high_speed_name)
                        .filter(|hs| {
                            let can_use = if let Some((ref m, l)) = hs.module_requirement {
                                module_levels.can_use(m, l)
                            } else { true };
                            can_use && facility_counts.can_produce(&hs.facility, hs.facility_level)
                        })
                        .or_else(|| item_map.get(raw_mat.as_str()));
                    
                    if let Some(mat) = mat_item {
                        if mat.raw_materials.is_some() {
                            // This is a processed intermediate
                            all_intermediate_steps.push((
                                mat.name.clone(),
                                mat.facility.clone(),
                                required,
                            ));
                        }
                    }
                }

                if skip_item {
                    continue;
                }

                let processing_facility_count = facility_counts.get_count(&item.facility) as f64;
                let processing_time_per_mill = item.production_time; // Time for 1 mill to process 1 batch
                let processing_time = processing_time_per_mill / processing_facility_count;

                // For steady-state production, we need to find the bottleneck between:
                // 1. Raw material production rate
                // 2. Processing rate
                //
                // Processing rate: processing_facility_count batches per processing_time_per_mill seconds
                // Raw material rate: depends on how fast we can gather ingredients
                //
                // max_ingredient_time is the time to gather materials for 1 processed batch
                // But with more farms, we gather materials faster than needed for 1 batch
                // 
                // For super_wheatmeal example:
                // - 1 batch needs 120 wheat
                // - With 20 farms, we gather 300 wheat per 90 seconds
                // - That's 300/120 = 2.5 batches worth per 90 seconds
                // - Processing can do 5 batches per 60 seconds = 7.5 batches per 90 seconds
                // - Bottleneck is gathering: 2.5 batches per 90 seconds
                //
                // The issue is max_ingredient_time is calculated assuming we stop after gathering 120 wheat.
                // We need to calculate the raw material RATE instead.
                //
                // For now, let's calculate: what's the rate at which farms can supply materials?
                // rate = (yield per batch × facility_count) / batch_time / required_per_processed_batch
                // This gives us "processed batches worth of materials per second"
                
                // Collect raw material details (for the same-facility multi-material allocation
                // feature, e.g. dried_flowers' lavender+rose both from Farmland) and fertilizer
                // requirements. The actual bottleneck-limited rate is computed separately below,
                // via the whole-tree resource-demand walk — NOT per-ingredient here — because two
                // ingredients can independently need the same deeper raw material (see
                // `accumulate_demand`'s doc comment), which a per-ingredient loop can't detect.
                for (i, raw_mat) in raw_mats.iter().enumerate() {
                    let required_per_batch = required_amounts.get(i).copied().unwrap_or(1) as f64;
                    let Some(raw) =
                        resolve_raw_material(raw_mat, &item_map, facility_counts, module_levels)
                    else {
                        continue;
                    };

                    // Collect raw material details for optimal allocation calculation
                    // Only include materials from the primary facility (for allocation to make sense)
                    // (name, amount_per_batch, time_per_batch, facility)
                    raw_material_details_collected.push((
                        raw.name.clone(),
                        required_per_batch as u32,
                        raw.production_time,
                        raw.facility.clone(),
                    ));

                    // Track fertilizer requirements
                    // Each batch of a fertilizer-requiring crop needs 1 fertilizer
                    // We need ceil(required_per_batch / yield) batches of raw material
                    if raw.requires_fertilizer {
                        let raw_batches_needed = (required_per_batch / raw.yield_amount as f64).ceil() as u32;
                        fertilizer_per_batch += raw_batches_needed;
                    }
                }

                // True bottleneck-limited rate for this item: walks the whole ingredient tree and
                // takes the minimum capacity/demand ratio over every facility touched anywhere in
                // it (including this item's own processing facility, and any raw material shared
                // across multiple branches) — see `compute_resource_demand`/`batch_rate_bound`.
                let demand = compute_resource_demand(item, &item_map, facility_counts, module_levels);
                let batches_per_second = batch_rate_bound(&demand, facility_counts);
                let facility_demand: Vec<(String, f64, Vec<String>)> = demand
                    .into_iter()
                    .map(|(facility, (utilization, items))| {
                        (
                            facility.to_string(),
                            utilization,
                            items.into_iter().map(|i| i.name.clone()).collect(),
                        )
                    })
                    .collect();

                let steady_state_time = if batches_per_second > 0.0 && batches_per_second.is_finite() {
                    1.0 / batches_per_second
                } else {
                    f64::INFINITY 
                };
                
                // Total time for a single batch (used for display) is still sequential
                let total_time = max_ingredient_time + processing_time;
                let total_energy = match (total_ingredient_energy, item.energy) {
                    (Some(ie), Some(pe)) => Some(ie + pe),
                    (Some(ie), None) => Some(ie),
                    (None, Some(pe)) => Some(pe),
                    (None, None) => None,
                };

                // Deduplicate raw names while preserving order
                let unique_raw_names: Vec<String> = {
                    let mut seen = HashSet::new();
                    all_raw_names.into_iter().filter(|n| seen.insert(n.clone())).collect()
                };
                
                // Only keep raw_material_details if we have multiple materials FROM THE SAME FACILITY
                // (allocation only makes sense when splitting facilities of the same type)
                let raw_details = if raw_material_details_collected.len() > 1 {
                    // Check if all materials come from the same facility
                    let first_facility = &raw_material_details_collected[0].3;
                    let all_same_facility = raw_material_details_collected.iter()
                        .all(|(_, _, _, facility)| facility == first_facility);
                    
                    if all_same_facility {
                        // Convert to (name, amount, time) format - drop facility field
                        Some(raw_material_details_collected.into_iter()
                            .map(|(name, amt, time, _)| (name, amt, time))
                            .collect())
                    } else {
                        // Materials come from different facilities, allocation doesn't apply
                        None
                    }
                } else {
                    None
                };

                (
                    total_time,
                    steady_state_time, // Pass steady_state_time for efficiency calculation
                    total_energy,
                    total_ingredient_cost,
                    Some(unique_raw_names.join("+")),
                    primary_facility,
                    all_facilities_collected,
                    all_intermediate_steps,
                    raw_details,
                    fertilizer_per_batch,
                    facility_demand,
                )
            } else {
                // This is a raw material - direct production
                let facility_count = facility_counts.get_count(&item.facility) as f64;
                let time_per_batch = item.production_time;
                
                // Add fertilizer time if required (1 fertilizer per batch)
                let fertilizer_time = if item.requires_fertilizer {
                    fertilizer_time_per_unit
                } else {
                    0.0
                };
                
                // For display purposes, time_per_unit is how long to produce one unit
                let effective_time_per_yield =
                    (time_per_batch + fertilizer_time) / (item.yield_amount as f64 * facility_count);
                // For raw materials, steady-state time equals batch time / facility count
                let steady_state_time = (time_per_batch + fertilizer_time) / facility_count;
                // Energy per batch (not per unit) to match units_needed which counts batches
                let energy_per_batch = item.energy;
                let cost_per_batch = item.cost.unwrap_or(0.0);
                
                // Raw materials use just their own facility
                let mut raw_all_facilities = HashSet::new();
                raw_all_facilities.insert(item.facility.clone());
                
                // For raw materials, 1 fertilizer per batch if required
                let raw_fertilizer = if item.requires_fertilizer { 1u32 } else { 0u32 };

                (effective_time_per_yield, steady_state_time, energy_per_batch, cost_per_batch, None, None, raw_all_facilities, vec![], None, raw_fertilizer, vec![(item.facility.clone(), item.production_time, vec![item.name.clone()])])
            };

        let net_profit = item.sell_value * item.yield_amount as f64 - raw_cost;
        
        // For efficiency comparison, use steady-state time (bottleneck)
        let profit_per_second = if steady_state_time > 0.0 {
            net_profit / steady_state_time
        } else {
            0.0
        };
        let profit_per_energy = total_energy.map(|e| if e > 0.0 { net_profit / e } else { 0.0 });

        // For time optimization, use batch-based profit_per_second directly
        // (facility parallelism is already factored into batch_time)
        let effective_profit_per_second = profit_per_second;
        
        // Startup time is the time to produce the first batch (before steady-state begins)
        // This equals total_time for the first unit/batch
        let startup_time = total_time;

        efficiencies.push(ProductionEfficiency {
            item: item.clone(),
            profit_per_second,
            profit_per_energy,
            total_time_per_unit: total_time,
            total_energy_per_unit: total_energy,
            requires_raw,
            raw_cost,
            raw_facility,
            all_facilities,
            intermediate_steps,
            startup_time,
            effective_profit_per_second,
            raw_material_details,
            fertilizer_per_batch,
            facility_demand,
        });
    }

    efficiencies
}

/// Finds the optimal production path to achieve a target currency amount.
///
/// This function selects the most efficient production option based on
/// the optimization mode (time or energy) and calculates the complete
/// production path including raw material gathering.
///
/// # Arguments
///
/// * `efficiencies` - Pre-calculated efficiency metrics for all items
/// * `target_amount` - Target amount of currency to produce
/// * `optimize_energy` - If true, optimize for energy efficiency; otherwise optimize for time
/// * `energy_cost_per_min` - Cost of energy per minute (used when optimizing for time)
/// * `facility_counts` - Configuration for each facility (count and level)
///
/// # Returns
///
/// An `Option<ProductionPath>` containing the optimal path, or `None` if no valid path exists.
///
/// # Optimization Modes
///
/// - **Time optimization** (default): Maximizes profit per second, considering energy costs
/// - **Energy optimization**: Maximizes profit per energy unit consumed
///
/// # Example
///
/// ```no_run
/// use aniimax::optimizer::{calculate_efficiencies, find_best_production_path};
/// use aniimax::models::{FacilityCounts, ModuleLevels};
/// use aniimax::data::load_all_data;
/// use std::path::Path;
///
/// let items = load_all_data(Path::new("data")).unwrap();
/// let counts = FacilityCounts::from_pairs(&[
///     ("Farmland", 4, 3),        // 4 farmlands at level 3
///     ("Woodland", 1, 2),        // 1 woodland at level 2
///     ("Mineral Pile", 1, 1),    // 1 mineral pile at level 1
///     ("Carousel Mill", 2, 2),   // 2 carousel mills at level 2
///     ("Jukebox Dryer", 1, 1),
///     ("Crafting Table", 1, 1),
///     ("Nimbus Bed", 1, 1),      // 1 nimbus bed (Wool/Petals)
/// ]);
/// let modules = ModuleLevels::default();
///
/// let efficiencies = calculate_efficiencies(&items, "coins", &counts, &modules);
/// let path = find_best_production_path(&efficiencies, 5000.0, false, 0.0, &counts);
/// ```
pub fn find_best_production_path(
    efficiencies: &[ProductionEfficiency],
    target_amount: f64,
    optimize_energy: bool,
    energy_cost_per_min: f64,
    facility_counts: &FacilityCounts,
) -> Option<ProductionPath> {
    if efficiencies.is_empty() {
        return None;
    }

    // Sort by efficiency metric
    let mut sorted = efficiencies.to_vec();
    if optimize_energy {
        sorted.sort_by(|a, b| {
            let a_eff = a.profit_per_energy.unwrap_or(0.0);
            let b_eff = b.profit_per_energy.unwrap_or(0.0);
            b_eff
                .partial_cmp(&a_eff)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        // When optimizing for time, use effective profit per second (considers parallelization)
        sorted.sort_by(|a, b| {
            let a_energy_cost = a.total_energy_per_unit.unwrap_or(0.0) * energy_cost_per_min / 60.0;
            let b_energy_cost = b.total_energy_per_unit.unwrap_or(0.0) * energy_cost_per_min / 60.0;
            let a_net =
                a.effective_profit_per_second - (a_energy_cost / a.total_time_per_unit.max(1.0));
            let b_net =
                b.effective_profit_per_second - (b_energy_cost / b.total_time_per_unit.max(1.0));
            b_net
                .partial_cmp(&a_net)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Get the best option
    let best = &sorted[0];

    // Calculate how many units we need to produce
    let profit_per_unit = best.item.sell_value * best.item.yield_amount as f64 - best.raw_cost;
    let units_needed = (target_amount / profit_per_unit).ceil() as u32;

    let mut steps = Vec::new();

    // Get facility count for the main production
    let main_facility_count = facility_counts.get_count(&best.item.facility);
    let nimbus_bed_count = facility_counts.get_count("Nimbus Bed");

    // Add fertilizer production step if needed
    if best.fertilizer_per_batch > 0 && nimbus_bed_count > 0 {
        let total_fertilizer_needed = best.fertilizer_per_batch * units_needed;
        
        steps.push(ProductionStep {
            item_name: "fertilizer".to_string(),
            facility: format!("Nimbus Bed (x{})", nimbus_bed_count),
            quantity: total_fertilizer_needed,
            time: 0.0, // Time is included in total
            energy: None,
            profit_contribution: 0.0,
            chain_id: None,
            facility_allocation: None,
        });
    }

    // Add raw material step if needed
    if let Some(ref raw_name) = best.requires_raw {
        // Sum all required amounts for display purposes
        let raw_amount_needed = best.item.required_amount
            .as_ref()
            .map(|amounts| amounts.iter().sum::<u32>())
            .unwrap_or(1) * units_needed;
        let raw_facility = best.raw_facility.as_deref().unwrap_or("Unknown");
        let raw_facility_count = facility_counts.get_count(raw_facility);
        
        // Calculate optimal facility allocation for multi-material production
        let facility_allocation = if let Some(ref details) = best.raw_material_details {
            // details is Vec<(name, amount_per_batch, time_per_batch)>
            // We need to scale amounts by units_needed and calculate optimal facility split
            let materials_for_allocation: Vec<(String, u32, f64)> = details.iter()
                .map(|(name, amt_per_batch, time)| {
                    (name.clone(), amt_per_batch * units_needed, *time)
                })
                .collect();
            
            let allocation = calculate_optimal_facility_allocation(&materials_for_allocation, raw_facility_count);
            if allocation.len() > 1 {
                Some(allocation)
            } else {
                None
            }
        } else {
            None
        };
        
        steps.push(ProductionStep {
            item_name: raw_name.clone(),
            facility: format!("{} (x{})", raw_facility, raw_facility_count),
            quantity: raw_amount_needed,
            time: 0.0, // Time is included in total
            energy: None,
            profit_contribution: 0.0,
            chain_id: None,
            facility_allocation,
        });
        
        // Add intermediate processing steps (e.g., nuts for caramel_nut_chips)
        for (int_name, int_facility, int_amount_per_batch) in &best.intermediate_steps {
            let int_qty = int_amount_per_batch * units_needed;
            steps.push(ProductionStep {
                item_name: int_name.clone(),
                facility: format!("{} (x{})", int_facility, facility_counts.get_count(int_facility)),
                quantity: int_qty,
                time: 0.0,
                energy: None,
                profit_contribution: 0.0,
                chain_id: None,
                facility_allocation: None,
            });
        }
    }

    // Add production step
    steps.push(ProductionStep {
        item_name: best.item.name.clone(),
        facility: format!("{} (x{})", best.item.facility, main_facility_count),
        quantity: units_needed,
        time: best.total_time_per_unit * units_needed as f64,
        energy: best
            .total_energy_per_unit
            .map(|e| e * units_needed as f64),
        profit_contribution: profit_per_unit * units_needed as f64,
        chain_id: None,
        facility_allocation: None,
    });

    // Calculate actual time with parallelization
    // For processed items, use the steady-state calculation:
    //   time = units_needed * profit_per_unit / effective_profit_per_second
    // For raw materials, use direct batch calculation
    let total_time = if best.requires_raw.is_some() {
        // For processed items, effective_profit_per_second already accounts for bottleneck
        // time = profit_needed / profit_per_second
        units_needed as f64 * profit_per_unit / best.effective_profit_per_second
    } else {
        // For raw materials, units_needed is already the number of batches
        best.item.production_time * (units_needed as f64 / main_facility_count as f64).ceil()
    };

    let total_energy = best
        .total_energy_per_unit
        .map(|e| e * units_needed as f64);

    Some(ProductionPath {
        steps,
        total_time: total_time + best.startup_time, // Include startup delay
        startup_time: best.startup_time,
        total_energy,
        total_profit: profit_per_unit * units_needed as f64,
        currency: best.item.sell_currency.clone(),
        items_produced: units_needed * best.item.yield_amount,
        is_energy_self_sufficient: false,
        energy_items_produced: None,
        energy_item_name: None,
    })
}

/// Finds the optimal production path using cross-facility parallelization.
///
/// This function finds all production chains that can run simultaneously without
/// sharing any facilities. For example:
/// - Farmland → Carousel Mill (super_wheatmeal)
/// - Woodland → Crafting Table (wood_sculpture)  
/// - Nimbus Bed (wool)
/// All running in parallel since they use different facilities.
///
/// # Algorithm
///
/// Uses a greedy approach:
/// 1. Sort all items by profit per second
/// 2. Select the best item
/// 3. Find the next best item that doesn't share any facilities with selected items
/// 4. Repeat until no more non-conflicting items can be added
///
/// # Arguments
///
/// * `efficiencies` - Pre-calculated efficiency metrics for all items
/// * `target_amount` - Target amount of currency to produce
/// * `facility_counts` - Configuration for each facility (count and level)
///
/// # Returns
///
/// An `Option<ProductionPath>` containing the optimal parallel path, or `None` if no valid path exists.
pub fn find_parallel_production_path(
    efficiencies: &[ProductionEfficiency],
    target_amount: f64,
    facility_counts: &FacilityCounts,
) -> Option<ProductionPath> {
    if efficiencies.is_empty() {
        return None;
    }

    // Helper to get all facilities used by an item (including intermediate processing)
    fn get_facilities_used(eff: &ProductionEfficiency) -> HashSet<String> {
        // Use the pre-computed all_facilities set which tracks the entire chain
        eff.all_facilities.clone()
    }

    // Sort efficiencies by profit per second (descending)
    let mut sorted_effs: Vec<&ProductionEfficiency> = efficiencies.iter().collect();
    sorted_effs.sort_by(|a, b| {
        b.effective_profit_per_second
            .partial_cmp(&a.effective_profit_per_second)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Greedily select non-conflicting items
    let mut selected_items: Vec<&ProductionEfficiency> = Vec::new();
    let mut occupied_facilities: HashSet<String> = HashSet::new();

    for eff in &sorted_effs {
        // Skip items with no profit
        if eff.effective_profit_per_second <= 0.0 {
            continue;
        }
        
        // Skip items from facilities with 0 count
        if facility_counts.get_count(&eff.item.facility) == 0 {
            continue;
        }
        if let Some(ref raw_fac) = eff.raw_facility {
            if facility_counts.get_count(raw_fac) == 0 {
                continue;
            }
        }

        let facilities_needed = get_facilities_used(eff);
        
        // Check if any facility is already occupied
        let has_conflict = facilities_needed.iter().any(|f| occupied_facilities.contains(f));
        
        if !has_conflict {
            // Add this item to selected list
            selected_items.push(eff);
            occupied_facilities.extend(facilities_needed);
        }
    }

    // Need at least 2 items for parallel mode to be useful
    if selected_items.len() <= 1 {
        return None;
    }

    // Calculate combined profit rate
    let combined_profit_per_second: f64 = selected_items
        .iter()
        .map(|eff| eff.effective_profit_per_second)
        .sum();

    // Calculate startup time: max first-batch time across all parallel chains
    // This is the time before steady-state production begins
    let startup_time: f64 = selected_items
        .iter()
        .map(|eff| eff.startup_time)
        .fold(0.0, f64::max);

    // Calculate time needed (steady-state only, startup added separately)
    let theoretical_time = target_amount / combined_profit_per_second;

    // Build production steps
    let mut steps = Vec::new();
    let mut total_profit = 0.0;
    let mut total_energy: Option<f64> = None;
    let mut total_items = 0u32;
    let mut chain_id: u32 = 0;

    for eff in &selected_items {
        let current_chain_id = chain_id;
        chain_id += 1;
        
        let profit_per_batch = eff.item.sell_value * eff.item.yield_amount as f64 - eff.raw_cost;
        
        // Calculate batches based on steady-state time
        let batches = if eff.requires_raw.is_some() {
            // Processed item: use steady-state calculation
            (theoretical_time * eff.effective_profit_per_second / profit_per_batch).ceil() as u32
        } else {
            // Raw item
            let facility_count = facility_counts.get_count(&eff.item.facility) as f64;
            let time_per_effective_batch = eff.item.production_time / facility_count;
            (theoretical_time / time_per_effective_batch).ceil() as u32
        };

        if batches == 0 {
            continue;
        }

        let step_profit = profit_per_batch * batches as f64;
        total_profit += step_profit;

        // Calculate actual time for this step
        let step_time = if eff.requires_raw.is_some() {
            // For processed items, time = batches * steady_state_time_per_batch
            batches as f64 * (profit_per_batch / eff.effective_profit_per_second)
        } else {
            let facility_count = facility_counts.get_count(&eff.item.facility) as f64;
            eff.item.production_time * (batches as f64 / facility_count).ceil()
        };

        if let Some(energy) = eff.total_energy_per_unit {
            let step_energy = energy * batches as f64;
            total_energy = Some(total_energy.unwrap_or(0.0) + step_energy);
        }

        total_items += batches * eff.item.yield_amount;
        
        let nimbus_bed_count = facility_counts.get_count("Nimbus Bed");
        
        // Add fertilizer production step if needed
        if eff.fertilizer_per_batch > 0 && nimbus_bed_count > 0 {
            let total_fertilizer_needed = eff.fertilizer_per_batch * batches;
            
            steps.push(ProductionStep {
                item_name: "fertilizer".to_string(),
                facility: format!("Nimbus Bed (x{})", nimbus_bed_count),
                quantity: total_fertilizer_needed,
                time: 0.0, // Time is included in total
                energy: None,
                profit_contribution: 0.0,
                chain_id: Some(current_chain_id),
                facility_allocation: None,
            });
        }

        // For processed items, show the full production chain
        if let Some(ref requires) = eff.requires_raw {
            // Step 1: Raw materials
            let raw_facility = eff.raw_facility.as_ref().unwrap_or(&eff.item.facility);
            let raw_qty = if let Some(ref amounts) = eff.item.required_amount {
                amounts.iter().sum::<u32>() * batches
            } else {
                batches
            };
            
            // Calculate optimal facility allocation for multi-material production
            let raw_facility_count = facility_counts.get_count(raw_facility);
            let facility_allocation = if let Some(ref details) = eff.raw_material_details {
                let materials_for_allocation: Vec<(String, u32, f64)> = details.iter()
                    .map(|(name, amt_per_batch, time)| {
                        (name.clone(), amt_per_batch * batches, *time)
                    })
                    .collect();
                
                let allocation = calculate_optimal_facility_allocation(&materials_for_allocation, raw_facility_count);
                if allocation.len() > 1 {
                    Some(allocation)
                } else {
                    None
                }
            } else {
                None
            };
            
            steps.push(ProductionStep {
                item_name: requires.clone(),
                facility: format!("{} (x{})", raw_facility, facility_counts.get_count(raw_facility)),
                quantity: raw_qty,
                time: step_time,
                energy: None,
                profit_contribution: 0.0,
                chain_id: Some(current_chain_id),
                facility_allocation,
            });
            
            // Step 2: Intermediate processing steps (e.g., nuts for caramel_nut_chips)
            for (int_name, int_facility, int_amount_per_batch) in &eff.intermediate_steps {
                let int_qty = int_amount_per_batch * batches;
                steps.push(ProductionStep {
                    item_name: int_name.clone(),
                    facility: format!("{} (x{})", int_facility, facility_counts.get_count(int_facility)),
                    quantity: int_qty,
                    time: step_time,
                    energy: None,
                    profit_contribution: 0.0,
                    chain_id: Some(current_chain_id),
                    facility_allocation: None,
                });
            }
        }

        // Step 3 (or 1 for raw items): Final product
        steps.push(ProductionStep {
            item_name: eff.item.name.clone(),
            facility: format!("{} (x{})", eff.item.facility, facility_counts.get_count(&eff.item.facility)),
            quantity: batches,
            time: step_time,
            energy: eff.total_energy_per_unit.map(|e| e * batches as f64),
            profit_contribution: step_profit,
            chain_id: Some(current_chain_id),
            facility_allocation: None,
        });
    }

    // Make sure we meet target by iteratively increasing if needed
    while total_profit < target_amount {
        // Find the step with highest profit/sec and add one batch
        let best_step_idx = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.profit_contribution > 0.0)
            .max_by(|(_, a), (_, b)| {
                let a_rate = a.profit_contribution / a.time;
                let b_rate = b.profit_contribution / b.time;
                a_rate.partial_cmp(&b_rate).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);

        if let Some(idx) = best_step_idx {
            let step = &mut steps[idx];
            let profit_per_batch = step.profit_contribution / step.quantity as f64;
            step.quantity += 1;
            step.profit_contribution += profit_per_batch;
            total_profit += profit_per_batch;
        } else {
            break;
        }
    }

    // Recalculate actual total time (longest step since they run in parallel)
    let actual_total_time = steps.iter().map(|s| s.time).fold(0.0, f64::max);

    // Only return if we have multiple independent productions
    let production_count = steps.iter().filter(|s| s.profit_contribution > 0.0).count();
    if production_count <= 1 {
        return None;
    }

    Some(ProductionPath {
        steps,
        total_time: actual_total_time + startup_time, // Include startup delay
        startup_time,
        total_energy,
        total_profit,
        currency: selected_items[0].item.sell_currency.clone(),
        items_produced: total_items,
        is_energy_self_sufficient: false,
        energy_items_produced: None,
        energy_item_name: None,
    })
}

/// Calculates efficiency metrics for items that can be consumed for energy.
///
/// Only items with a non-None energy field can be consumed for energy.
/// This is used for energy self-sufficient mode.
pub fn calculate_energy_efficiencies(
    items: &[ProductionItem],
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
) -> Vec<EnergyItemEfficiency> {
    let mut efficiencies = Vec::new();

    for item in items {
        // Only raw materials (no raw_materials field) can be efficiently produced for energy
        // Processed items require raw materials which have opportunity cost
        if item.raw_materials.is_some() {
            continue;
        }

        // Must have energy value to be consumable
        let energy_per_batch = match item.energy {
            Some(e) if e > 0.0 => e,
            _ => continue,
        };

        // Filter by facility level
        if !facility_counts.can_produce(&item.facility, item.facility_level) {
            continue;
        }

        // Filter by module requirement
        if let Some((ref module_name, required_level)) = item.module_requirement {
            if !module_levels.can_use(module_name, required_level) {
                continue;
            }
        }

        let facility_count = facility_counts.get_count(&item.facility) as f64;
        let time_per_batch = item.production_time / facility_count;
        let energy_per_second = energy_per_batch / time_per_batch;
        let cost_per_batch = item.cost.unwrap_or(0.0);

        efficiencies.push(EnergyItemEfficiency {
            item: item.clone(),
            energy_per_second,
            time_per_batch,
            energy_per_batch,
            cost_per_batch,
        });
    }

    // Sort by energy per second (best first)
    efficiencies.sort_by(|a, b| {
        b.energy_per_second
            .partial_cmp(&a.energy_per_second)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    efficiencies
}

/// Finds the optimal production path with energy self-sufficiency.
///
/// This function calculates a production plan where:
/// - Some facilities produce items for profit (to sell)
/// - Some facilities produce items for energy (to consume)
/// - Total energy from consumed items >= energy consumed during production
///
/// # Arguments
///
/// * `profit_efficiencies` - Efficiency metrics for profit items
/// * `energy_efficiencies` - Efficiency metrics for energy items
/// * `target_amount` - Target profit to achieve
/// * `energy_cost_per_min` - Energy consumed per minute of production
/// * `facility_counts` - Configuration for each facility
///
/// # Returns
///
/// An `Option<ProductionPath>` with the optimal self-sufficient plan.
pub fn find_self_sufficient_path(
    profit_efficiencies: &[ProductionEfficiency],
    energy_efficiencies: &[EnergyItemEfficiency],
    target_amount: f64,
    energy_cost_per_min: f64,
    facility_counts: &FacilityCounts,
) -> Option<ProductionPath> {
    if profit_efficiencies.is_empty() {
        return None;
    }

    // If no energy cost, just use the simple path
    if energy_cost_per_min <= 0.0 {
        return find_best_production_path(
            profit_efficiencies,
            target_amount,
            false,
            0.0,
            facility_counts,
        );
    }

    // If no energy items available, can't be self-sufficient
    if energy_efficiencies.is_empty() {
        return None;
    }

    // Sort profit items by profit per second
    let mut sorted_profit = profit_efficiencies.to_vec();
    sorted_profit.sort_by(|a, b| {
        b.effective_profit_per_second
            .partial_cmp(&a.effective_profit_per_second)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let best_profit = &sorted_profit[0];
    let best_energy = &energy_efficiencies[0]; // Already sorted

    // Calculate profit per batch for the profit item
    let profit_per_batch = best_profit.item.sell_value * best_profit.item.yield_amount as f64
        - best_profit.raw_cost;

    // Get facility counts
    let profit_facility_count = facility_counts.get_count(&best_profit.item.facility) as f64;
    let energy_facility_count = facility_counts.get_count(&best_energy.item.facility) as f64;

    // Energy rate (per second)
    let energy_rate = energy_cost_per_min / 60.0;

    // Energy production rate from the best energy item (with all facilities)
    let energy_production_rate = best_energy.energy_per_second * energy_facility_count;

    // Check if we can even be self-sufficient
    // We need: energy_production_rate > energy_rate (otherwise we can never catch up)
    if energy_production_rate <= energy_rate {
        // Can't be self-sufficient with current setup
        return None;
    }

    // Calculate the optimal split
    // Let T_profit = time producing profit items
    // Let T_energy = time producing energy items
    // Let T_total = T_profit + T_energy
    //
    // Constraints:
    // 1. Profit >= target: profit_rate * T_profit >= target
    // 2. Energy balance: energy_produced >= energy_consumed
    //    best_energy.energy_per_second * energy_facility_count * T_energy >= energy_rate * T_total
    //
    // From constraint 2:
    // E * T_energy >= R * (T_profit + T_energy)
    // E * T_energy >= R * T_profit + R * T_energy
    // T_energy * (E - R) >= R * T_profit
    // T_energy >= T_profit * R / (E - R)

    // Calculate time needed for profit production
    let batches_for_profit = (target_amount / profit_per_batch).ceil();

    // Time to produce profit items (with parallelization)
    let time_for_profit = if best_profit.requires_raw.is_some() {
        best_profit.total_time_per_unit * batches_for_profit / profit_facility_count
    } else {
        best_profit.item.production_time * (batches_for_profit / profit_facility_count).ceil()
    };

    // Calculate energy batches needed using the formula:
    // Energy needed = (T_profit + T_energy) * R
    // Energy produced = B * E  (where B = batches, E = energy per batch)
    // T_energy = production_time * ceil(B / facility_count)
    //
    // For self-sufficiency: B * E >= (T_profit + T_energy) * R
    // 
    // Let's solve iteratively since ceiling makes it non-linear
    let production_time_per_batch = best_energy.item.production_time;
    
    // Start with minimum batches and increase until we have enough energy
    let mut energy_batches = 1u32;
    loop {
        let rounds = (energy_batches as f64 / energy_facility_count).ceil();
        let actual_energy_time = production_time_per_batch * rounds;
        let total_time = time_for_profit + actual_energy_time;
        let energy_needed = total_time * energy_rate;
        let energy_produced = energy_batches as f64 * best_energy.energy_per_batch;
        
        if energy_produced >= energy_needed {
            break;
        }
        
        energy_batches += 1;
        
        // Safety check to prevent infinite loop
        if energy_batches > 10000 {
            return None;
        }
    }

    // Calculate actual times with the determined batch counts
    let energy_rounds = (energy_batches as f64 / energy_facility_count).ceil();
    let actual_energy_production_time = production_time_per_batch * energy_rounds;
    let total_time = time_for_profit + actual_energy_production_time;
    let total_energy_needed = total_time * energy_rate;

    // Build the production steps
    let mut steps = Vec::new();

    // Add energy production step
    steps.push(ProductionStep {
        item_name: format!("{} (for energy)", best_energy.item.name),
        facility: format!(
            "{} (x{})",
            best_energy.item.facility,
            facility_counts.get_count(&best_energy.item.facility)
        ),
        quantity: energy_batches,
        time: actual_energy_production_time,
        energy: Some(energy_batches as f64 * best_energy.energy_per_batch),
        profit_contribution: -(energy_batches as f64 * best_energy.cost_per_batch), // Cost of seeds
        chain_id: None,
        facility_allocation: None,
    });

    let nimbus_bed_count = facility_counts.get_count("Nimbus Bed");
    
    // Add fertilizer production step if needed for profit item
    if best_profit.fertilizer_per_batch > 0 && nimbus_bed_count > 0 {
        let total_fertilizer_needed = best_profit.fertilizer_per_batch * batches_for_profit as u32;
        
        steps.push(ProductionStep {
            item_name: "fertilizer".to_string(),
            facility: format!("Nimbus Bed (x{})", nimbus_bed_count),
            quantity: total_fertilizer_needed,
            time: 0.0, // Time is included in total
            energy: None,
            profit_contribution: 0.0,
            chain_id: None,
            facility_allocation: None,
        });
    }

    // Add raw material step for profit item if needed
    if let Some(ref raw_name) = best_profit.requires_raw {
        let raw_amount_needed = best_profit.item.required_amount
            .as_ref()
            .map(|amounts| amounts.iter().sum::<u32>())
            .unwrap_or(1) * batches_for_profit as u32;
        let raw_facility = best_profit.raw_facility.as_deref().unwrap_or("Unknown");
        let raw_facility_count = facility_counts.get_count(raw_facility);
        
        // Calculate optimal facility allocation for multi-material production
        let facility_allocation = if let Some(ref details) = best_profit.raw_material_details {
            let materials_for_allocation: Vec<(String, u32, f64)> = details.iter()
                .map(|(name, amt_per_batch, time)| {
                    (name.clone(), amt_per_batch * batches_for_profit as u32, *time)
                })
                .collect();
            
            let allocation = calculate_optimal_facility_allocation(&materials_for_allocation, raw_facility_count);
            if allocation.len() > 1 {
                Some(allocation)
            } else {
                None
            }
        } else {
            None
        };
        
        steps.push(ProductionStep {
            item_name: raw_name.clone(),
            facility: format!("{} (x{})", raw_facility, raw_facility_count),
            quantity: raw_amount_needed,
            time: 0.0,
            energy: None,
            profit_contribution: 0.0,
            chain_id: None,
            facility_allocation,
        });
        
        // Add intermediate processing steps (e.g., nuts for caramel_nut_chips)
        for (int_name, int_facility, int_amount_per_batch) in &best_profit.intermediate_steps {
            let int_qty = int_amount_per_batch * batches_for_profit as u32;
            steps.push(ProductionStep {
                item_name: int_name.clone(),
                facility: format!("{} (x{})", int_facility, facility_counts.get_count(int_facility)),
                quantity: int_qty,
                time: 0.0,
                energy: None,
                profit_contribution: 0.0,
                chain_id: None,
                facility_allocation: None,
            });
        }
    }

    // Add profit production step
    steps.push(ProductionStep {
        item_name: format!("{} (for profit)", best_profit.item.name),
        facility: format!(
            "{} (x{})",
            best_profit.item.facility,
            facility_counts.get_count(&best_profit.item.facility)
        ),
        quantity: batches_for_profit as u32,
        time: time_for_profit,
        energy: None,
        profit_contribution: profit_per_batch * batches_for_profit,
        chain_id: None,
        facility_allocation: None,
    });

    // Calculate actual profit (minus seed costs for energy items)
    let energy_seed_cost = energy_batches as f64 * best_energy.cost_per_batch;
    let gross_profit = profit_per_batch * batches_for_profit;
    let net_profit = gross_profit - energy_seed_cost;
    
    // For energy self-sufficient mode, startup time is the longer of the two chains
    let startup_time = best_profit.startup_time.max(best_energy.item.production_time);

    Some(ProductionPath {
        steps,
        total_time: total_time + startup_time,
        startup_time,
        total_energy: Some(total_energy_needed),
        total_profit: net_profit,
        currency: best_profit.item.sell_currency.clone(),
        items_produced: batches_for_profit as u32 * best_profit.item.yield_amount,
        is_energy_self_sufficient: true,
        energy_items_produced: Some(energy_batches * best_energy.item.yield_amount),
        energy_item_name: Some(best_energy.item.name.clone()),
    })
}

/// Finds the fastest way to reach a target coin balance, given the current balance, by running
/// a conflict-free set of coin-producing items simultaneously across every owned facility.
///
/// # Approach
///
/// Greedy set-packing by rate, same pattern as [`find_parallel_production_path`]: sort every
/// candidate item (across all facilities, not just one per facility) by
/// `effective_profit_per_second` descending, then walk the list claiming facilities as items are
/// selected. An item is only selected if EVERY facility its production chain touches
/// (`eff.all_facilities` — its own facility plus any raw-material/intermediate facilities) is
/// still unclaimed; selecting it claims all of them.
///
/// This matters because `calculate_efficiencies` computes a processed item's rate assuming the
/// *entire* owned count of its raw-material facility is dedicated to gathering that ingredient
/// (see its "gathering rate" comment) — e.g. Bouncy Brew Keg's rice_drink assumes all of
/// Farmland is growing rice for it. Picking the best item independently per facility (the
/// previous approach) ignored this: Farmland could simultaneously be told to grow strawberries
/// (its own best standalone item) *and* be silently assumed to supply rice_drink's rice, which
/// isn't physically possible with one Farmland. The greedy claim step prevents that: once
/// rice_drink claims Farmland, Farmland's own standalone candidates (strawberries, etc.) are
/// skipped since Farmland is no longer free.
///
/// Completion time is NOT simply `delta_coins / total_rate` — that would assume every selected
/// item is already at steady-state output from t=0, ignoring that a processed item's ingredients
/// have to actually grow/gather before the first batch can even be processed (rice_drink can't
/// sell anything until rice has grown *and* been through Bouncy Brew Keg once). Each selected
/// item instead contributes `rate * max(0, t - lead_time)`: nothing until its own first-batch
/// lead time (`item_lead_time`) has passed, then steady-state income after that — a hybrid of
/// "everything starts at once" (still true — every facility begins working in parallel at t=0)
/// and "nothing is instant" (each item's income stream is delayed by its own pipeline depth, not
/// by everyone else's). Coins accumulated by time `t` is the sum of that across every selected
/// item, which is monotonically non-decreasing and piecewise-linear in `t`, so the minimal `t`
/// reaching `delta_coins` is found by binary search (same doubling-then-bisecting pattern used
/// by the removed multi-resource version of this function — see history below). Since using more
/// facilities always helps (never hurts) reach the target sooner, the minimal plan still uses
/// every claimable facility for the full duration.
///
/// # History
///
/// This originally optimized for coins + Wood Blocks + Mineral Sand simultaneously (an RV/
/// Homeland level-up costs all three), with Wood Blocks/Mineral Sand production facilities
/// dedicated to byproduct income and a Resource Exchange integration to cover shortfalls. That
/// made some facilities' coin output "extra" (already covered by byproduct side-income from a
/// long Wood-Blocks-driven duration), which is where the `NotNeeded` status on `PlanStep`
/// came from. Dropped in `BETA_NOTES.md` section 30 — Wood Blocks/Mineral Sand are trivially
/// obtained by expanding plots in-game, so they weren't worth optimizing for. Section 31 then
/// replaced the naive best-per-facility selection with the greedy claim-based approach described
/// above, after it was caught recommending a facility for its own best item while also assuming
/// (elsewhere) that the same facility fully supplied another item's ingredients. See sections 23,
/// 27, and 29 for the full history of the removed multi-resource design.
/// Time (seconds) until the very first batch of `name` is ready — the growing/gathering lead
/// time before any output (and thus any coin income) exists at all, as opposed to the
/// steady-state throughput rate used everywhere else in this module.
///
/// For a raw material this is just `production_time` (every owned copy of the facility starts
/// at t=0 and finishes its first batch together, regardless of facility count — more facilities
/// mean more *batches per completion*, not a faster *first* completion). For a processed item
/// it's the slowest ingredient's own lead time, plus this item's own processing time on top
/// (ingredients are gathered in parallel with each other, but processing can't start on an
/// ingredient before that ingredient exists). Recurses for multi-level chains (e.g. nuts, itself
/// processed, used as an ingredient in caramel_nut_chips).
///
/// Deliberately ignores facility count and the fertilizer add-on time (both already folded into
/// `ProductionEfficiency::startup_time`, which this does NOT reuse — that field divides
/// processing time by facility count, which is right for steady-state throughput but wrong for
/// "time until the first batch exists", the thing this function needs).
fn item_lead_time(name: &str, item_map: &HashMap<&str, &ProductionItem>, depth: u32) -> f64 {
    if depth > 8 {
        return 0.0; // guard against unexpected circular references
    }
    let Some(item) = item_map.get(name) else {
        return 0.0;
    };
    match &item.raw_materials {
        None => item.production_time,
        Some(raw_mats) => {
            let max_ingredient_lead = raw_mats
                .iter()
                .map(|m| item_lead_time(m, item_map, depth + 1))
                .fold(0.0_f64, f64::max);
            max_ingredient_lead + item.production_time
        }
    }
}

/// Solves for the provably-optimal simultaneous allocation of every owned facility's capacity
/// across every candidate item — replacing the old greedy-plus-leftover-patches approach (three
/// separate mechanisms accumulated over this project's history, each added to fix one more
/// scenario the previous ones missed) with a linear program: maximize total coins/sec subject to
/// no facility's capacity being oversubscribed.
///
/// `eff.facility_demand` (see `compute_resource_demand`) already gives exactly the constraint
/// coefficients needed: for item `i` and facility `f`, how much of `f`'s batch capacity one
/// batch/sec of `i` consumes. That's the entire LP — one variable per item (its batches/sec,
/// coefficient = net profit per batch, ≥ 0), one constraint per owned facility (sum of
/// utilization × rate across every item touching it ≤ that facility's count). A facility ending
/// up split between multiple items falls out of the solution naturally whenever that's optimal —
/// no separate "leftover" bookkeeping needed, and no dependency on the order items happen to be
/// considered in, since every variable is solved for simultaneously.
///
/// Returns batches/sec for every item whose solved rate is above a negligible threshold, keyed by
/// item name.
fn solve_facility_allocation<'a>(
    effs: &'a [ProductionEfficiency],
    facility_counts: &FacilityCounts,
) -> HashMap<&'a str, f64> {
    let mut problem = Problem::new(OptimizationDirection::Maximize);

    // One column per candidate item with positive net profit — anything else can never help the
    // objective (its coefficient would be ≤ 0), so excluding it up front keeps the problem small
    // without changing the optimal solution.
    let mut item_vars: Vec<(&'a ProductionEfficiency, microlp::Variable)> = Vec::new();
    for eff in effs {
        if facility_counts.get_count(&eff.item.facility) == 0 {
            continue;
        }
        let net_profit_per_batch = eff.item.sell_value * eff.item.yield_amount as f64 - eff.raw_cost;
        if net_profit_per_batch <= 0.0 {
            continue;
        }
        let var = problem.add_var(net_profit_per_batch, (0.0, f64::INFINITY));
        item_vars.push((eff, var));
    }

    // One row per owned facility touched by at least one candidate: total utilization across
    // every item using it can't exceed its batch capacity.
    let mut facility_names: HashSet<&str> = HashSet::new();
    for (eff, _) in &item_vars {
        for (facility, _, _) in &eff.facility_demand {
            facility_names.insert(facility.as_str());
        }
    }
    for facility in facility_names {
        // No lower bound skip here on purpose: a facility you own zero of must still get a
        // constraint (capacity 0), not no constraint at all — skipping it here would let the LP
        // treat "you don't own this" as "unlimited supply of it" instead of "none available".
        let capacity = facility_counts.get_count(facility) as f64;
        let terms: Vec<(microlp::Variable, f64)> = item_vars
            .iter()
            .filter_map(|(eff, var)| {
                eff.facility_demand
                    .iter()
                    .find(|(f, _, _)| f == facility)
                    .map(|(_, utilization, _)| (*var, *utilization))
            })
            .collect();
        if terms.is_empty() {
            continue;
        }
        problem.add_constraint(&terms, ComparisonOp::Le, capacity);
    }

    // Every variable is bounded by at least its own facility's constraint (facility_demand always
    // includes the item's own facility — see `accumulate_demand`), and every constraint's RHS is
    // a non-negative facility count, so this should always be feasible and bounded. Degrade to
    // "nothing selected" rather than panic if that assumption is ever wrong.
    let Ok(solution) = problem.solve() else {
        return HashMap::new();
    };

    item_vars
        .into_iter()
        .filter_map(|(eff, var)| {
            let rate = solution[var];
            if rate > 1e-9 {
                Some((eff.item.name.as_str(), rate))
            } else {
                None
            }
        })
        .collect()
}

/// Converts fractional shares of a facility's capacity into whole facility counts, using the
/// largest-remainder method (the same apportionment technique used to allocate parliament seats)
/// so the rounded counts land as close as possible to the true fractions while still being
/// integers — a player assigns a whole plot to a crop, not a percentage of one, so "78% funnels
/// into X" isn't actionable the way "16 Farmland grow X" is. `fractions` need not sum to 1.0 (any
/// shortfall is genuinely idle capacity, not an artifact of rounding); the returned counts sum to
/// `round(fractions.sum() * total)`, not to `total` itself, for the same reason.
fn apportion_counts(fractions: &[f64], total: u32) -> Vec<u32> {
    let total_f = total as f64;
    let ideal: Vec<f64> = fractions.iter().map(|f| f * total_f).collect();
    let mut counts: Vec<u32> = ideal.iter().map(|v| v.floor() as u32).collect();
    let target = ideal.iter().sum::<f64>().round() as u32;
    let base: u32 = counts.iter().sum();
    let remaining = target.saturating_sub(base) as usize;

    let mut order: Vec<usize> = (0..ideal.len()).collect();
    order.sort_by(|&a, &b| {
        let remainder_a = ideal[a] - counts[a] as f64;
        let remainder_b = ideal[b] - counts[b] as f64;
        remainder_b.partial_cmp(&remainder_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    for &i in order.iter().take(remaining) {
        counts[i] += 1;
    }
    counts
}

/// A GROWER facility (Farmland, Woodland, Mineral Pile, ...) has every plot committed to one crop
/// for its whole cycle — it structurally never hosts a processed item (see the data loaders in
/// `data.rs`: `load_farmland`/`load_woodland`/`load_workload_raw_material`/`load_nimbus_bed`
/// always set `raw_materials: None`; `load_processing_*` always set it to `Some`). A PROCESSOR
/// facility (Claw Game Cooker, Carousel Mill, ...) has no such constraint — it processes whatever
/// ingredients are ready, so genuinely cycling between recipes in whatever proportion their
/// inputs allow is real, achievable behavior, not something that needs rounding.
fn is_grower_facility(items: &[ProductionItem], name: &str) -> bool {
    !items.iter().any(|it| it.facility == name && it.raw_materials.is_some())
}

/// Apportions every grower facility's continuous LP shares into authoritative whole-unit counts
/// (see `apportion_counts`), one facility at a time. Keyed by owned `String` (facility, item name)
/// rather than borrowing from `allocation`/`eff_by_name`, since this needs to be called against a
/// *trial* candidate set that may get discarded (see `find_production_plan`'s stranded-chain exclusion
/// loop) as well as the final settled one.
fn build_grower_assignment(
    items: &[ProductionItem],
    allocation: &HashMap<&str, f64>,
    eff_by_name: &HashMap<&str, &ProductionEfficiency>,
    facility_counts: &FacilityCounts,
) -> HashMap<(String, String), u32> {
    let mut grower_shares: HashMap<&str, Vec<(&str, f64)>> = HashMap::new();
    for (&name, &rate) in allocation {
        let eff = eff_by_name[name];
        for (facility, utilization, _) in &eff.facility_demand {
            if !is_grower_facility(items, facility) {
                continue;
            }
            let capacity = facility_counts.get_count(facility) as f64;
            if capacity <= 0.0 {
                continue;
            }
            let fraction = utilization * rate / capacity;
            if fraction <= 0.0 {
                continue;
            }
            grower_shares.entry(facility.as_str()).or_default().push((name, fraction));
        }
    }

    let mut grower_assignment: HashMap<(String, String), u32> = HashMap::new();
    for (&facility, shares) in &grower_shares {
        let fractions: Vec<f64> = shares.iter().map(|(_, f)| *f).collect();
        let counts = apportion_counts(&fractions, facility_counts.get_count(facility));
        for (&(item_name, _), &count) in shares.iter().zip(&counts) {
            grower_assignment.insert((facility.to_string(), item_name.to_string()), count);
        }
    }
    grower_assignment
}

/// Caps an item's continuous LP rate by what its grower facilities can ACTUALLY supply once
/// rounded to whole units (see `build_grower_assignment`). This is the only correction needed:
/// the continuous rate already correctly accounts for fair sharing at every PROCESSOR facility
/// (that's what solving the LP jointly rather than greedily buys us), so redoing that math
/// independently per item here — instead of taking the min against the untouched continuous
/// rate — would silently reintroduce the shared-resource double-counting bug fixed earlier this
/// session (e.g. Claw Game Cooker's three-way split would let each item assume exclusive access
/// to it again).
fn final_rate_for(
    items: &[ProductionItem],
    eff: &ProductionEfficiency,
    continuous_rate: f64,
    grower_assignment: &HashMap<(String, String), u32>,
) -> f64 {
    eff.facility_demand
        .iter()
        .filter(|(f, _, _)| is_grower_facility(items, f))
        .fold(continuous_rate, |bound, (facility, utilization, _)| {
            if *utilization <= 0.0 {
                return bound;
            }
            let assigned = grower_assignment
                .get(&(facility.clone(), eff.item.name.clone()))
                .copied()
                .unwrap_or(0);
            bound.min(assigned as f64 / utilization)
        })
}

/// For every PROCESSOR facility touched by any candidate item with a positive final rate, the
/// items using it and `(fraction of its capacity, rate_per_second)` — reused both to detect
/// "genuine contention" (more distinct recipes wanting a facility than it has units — see
/// `find_production_plan`'s exclusion loop, since a processor can only ever be dedicated to ONE
/// recipe at a time, not fractionally time-shared between several) and to build the final
/// `coin_items` facility-plan rows, so this logic isn't duplicated.
///
/// A useful invariant: for a facility used by only ONE item, that item's `fraction` is always
/// `≤ 1`, because the LP's own capacity constraint (`Σ utilization × rate ≤ capacity`) is defined
/// relative to that same `capacity` — a solo consumer's usage can never exceed what the
/// constraint allows. So every contributor to a contended facility needs exactly one dedicated
/// unit, never more.
fn build_processor_usage<'a>(
    items: &[ProductionItem],
    allocation: &HashMap<&'a str, f64>,
    eff_by_name: &HashMap<&'a str, &'a ProductionEfficiency>,
    facility_counts: &FacilityCounts,
    grower_assignment: &HashMap<(String, String), u32>,
) -> HashMap<&'a str, Vec<(&'a ProductionEfficiency, f64, f64)>> {
    let mut usage: HashMap<&str, Vec<(&ProductionEfficiency, f64, f64)>> = HashMap::new();
    for (&name, &continuous_rate) in allocation {
        let eff = eff_by_name[name];
        let rate = final_rate_for(items, eff, continuous_rate, grower_assignment);
        if rate <= 0.0 {
            continue;
        }
        let net_profit_per_batch = eff.item.sell_value * eff.item.yield_amount as f64 - eff.raw_cost;
        let rate_per_second = net_profit_per_batch * rate;
        for (facility, utilization, _) in &eff.facility_demand {
            if is_grower_facility(items, facility) {
                continue;
            }
            let capacity = facility_counts.get_count(facility) as f64;
            if capacity <= 0.0 {
                continue;
            }
            let fraction = utilization * rate / capacity;
            if fraction < 0.001 {
                continue; // negligible, not worth reporting
            }
            usage.entry(facility.as_str()).or_default().push((eff, fraction, rate_per_second));
        }
    }
    usage
}

/// Solves for the provably-optimal simultaneous use of every owned facility for one currency
/// (`"coins"` or `"bud_tickets"`, matching `calculate_efficiencies`' `target_currency`) —
/// target-independent: no goal amount is needed to know the best achievable rate and facility
/// plan. Pass the result to `time_to_reach_goal` to find out how long a specific goal takes.
pub fn find_production_plan(
    items: &[ProductionItem],
    currency: &str,
    facility_counts: &FacilityCounts,
    module_levels: &ModuleLevels,
) -> Option<ProductionPlan> {
    let item_map: HashMap<&str, &ProductionItem> =
        items.iter().map(|i| (i.name.as_str(), i)).collect();

    let effs = calculate_efficiencies(items, currency, facility_counts, module_levels);

    // Solve for the provably-optimal simultaneous allocation of every owned facility's capacity
    // across every candidate item — see `solve_facility_allocation`'s doc comment for why this
    // replaces the old greedy-plus-leftover-patches approach entirely.
    //
    // A chain needing TWO OR MORE different grower facilities (e.g. maple_candy_star needs both
    // Woodland's maple_syrup and Starfall Hammock's star) can get "stranded": each grower is
    // apportioned independently (`build_grower_assignment`), so it's possible for one of them to
    // round that chain's share all the way down to zero (its capacity going instead to something
    // more valuable) while the OTHER grower still shows a whole-unit assignment to a chain that
    // can now never actually produce anything. Detect that and re-solve with the dead chain
    // excluded, so the LP finds the stranded facility's genuinely useful alternative instead of
    // recommending a dedication to nothing — repeats until stable (each pass excludes at least
    // one more item, and the candidate set is small, so this converges quickly).
    let mut excluded: HashSet<String> = HashSet::new();
    let candidates: Vec<ProductionEfficiency> = loop {
        let trial: Vec<ProductionEfficiency> =
            effs.iter().filter(|e| !excluded.contains(&e.item.name)).cloned().collect();
        let trial_allocation = solve_facility_allocation(&trial, facility_counts);
        if trial_allocation.is_empty() {
            // Nothing profitable available anywhere — genuinely infeasible.
            return None;
        }
        let trial_eff_by_name: HashMap<&str, &ProductionEfficiency> =
            trial.iter().map(|e| (e.item.name.as_str(), e)).collect();
        let trial_growers =
            build_grower_assignment(items, &trial_allocation, &trial_eff_by_name, facility_counts);
        let stranded: Vec<String> = trial_allocation
            .iter()
            .filter(|&(&name, &rate)| {
                let eff = trial_eff_by_name[name];
                final_rate_for(items, eff, rate, &trial_growers) <= 0.0
            })
            .map(|(&name, _)| name.to_string())
            .collect();

        // A processor facility can only ever be "set and left" on ONE recipe at a time — the
        // continuous LP relaxation's assumption that it can be fractionally time-shared between
        // several recipes isn't something a player can actually execute. When more distinct
        // recipes want a facility than it has units, keep the `owned` most profitable ones
        // (ranked by their own rate_per_second — every contributor needs exactly one dedicated
        // unit regardless of its fraction, see `build_processor_usage`'s doc comment, so ranking
        // by economic value directly answers "which recipe is worth the unit") and exclude the
        // rest, re-solving so the LP finds their genuinely-usable alternative instead of
        // recommending an unexecutable fractional split.
        let trial_processor_usage = build_processor_usage(
            items,
            &trial_allocation,
            &trial_eff_by_name,
            facility_counts,
            &trial_growers,
        );
        let mut contention_losers: Vec<String> = Vec::new();
        for (&facility, contributors) in &trial_processor_usage {
            let owned = facility_counts.get_count(facility) as usize;
            if contributors.len() > owned {
                let mut sorted = contributors.clone();
                sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                contention_losers
                    .extend(sorted.iter().skip(owned).map(|(eff, _, _)| eff.item.name.clone()));
            }
        }

        if stranded.is_empty() && contention_losers.is_empty() {
            break trial;
        }
        excluded.extend(stranded);
        excluded.extend(contention_losers);
    };

    // `candidates` has now settled (no stranded chains) — solve once more against it so
    // `allocation`/`eff_by_name`/`grower_assignment` all borrow from a value that lives for the
    // rest of the function, instead of threading the loop's trial values out through the borrow
    // checker. Cheap: this problem size solves in well under a millisecond.
    let allocation = solve_facility_allocation(&candidates, facility_counts);
    let eff_by_name: HashMap<&str, &ProductionEfficiency> =
        candidates.iter().map(|e| (e.item.name.as_str(), e)).collect();
    let grower_assignment = build_grower_assignment(items, &allocation, &eff_by_name, facility_counts);
    let is_grower = |name: &str| is_grower_facility(items, name);
    let final_rate = |name: &str, continuous_rate: f64| -> f64 {
        final_rate_for(items, eff_by_name[name], continuous_rate, &grower_assignment)
    };

    // One income stream per item the LP actually chose to produce.
    let mut income_streams: Vec<PlanProduct> = Vec::new();
    for (&name, &continuous_rate) in &allocation {
        let rate = final_rate(name, continuous_rate);
        if rate <= 0.0 {
            continue; // fully squeezed out by grower rounding — no income from this item after all
        }
        let eff = eff_by_name[name];
        let net_profit_per_batch = eff.item.sell_value * eff.item.yield_amount as f64 - eff.raw_cost;
        income_streams.push(PlanProduct {
            item_name: eff.item.name.clone(),
            facility: eff.item.facility.clone(),
            sell_value: eff.item.sell_value,
            rate_per_second: net_profit_per_batch * rate,
            units_per_second: rate * eff.item.yield_amount as f64,
            lead_time: item_lead_time(&eff.item.name, &item_map, 0),
            total_units: 0.0,
            total_value: 0.0,
        });
    }

    // Facility -> every item using it, its fraction of capacity, and its rate_per_second —
    // replaces the old `claimed_by`/`facility_leftover`/`facility_feeding_item` trio with one
    // structure that naturally supports any number of contributors per facility. Only
    // meaningfully used for PROCESSOR facilities below — grower facilities are reported straight
    // from `grower_assignment` instead. By this point the exclusion loop above already guarantees
    // no processor facility has more contributors than owned units (see `build_processor_usage`'s
    // doc comment), so `coin_items` below never needs to fall back to describing an unexecutable
    // fractional time-share.
    let facility_usage =
        build_processor_usage(items, &allocation, &eff_by_name, facility_counts, &grower_assignment);

    let rate_per_second = income_streams.iter().map(|p| p.rate_per_second).sum();

    // Every distinct facility the user owns (count > 0), so the result can report on ALL owned
    // facilities — including ones with nothing profitable to produce right now, not just the
    // productive ones.
    let mut facility_names: Vec<&str> = items
        .iter()
        .map(|i| i.facility.as_str())
        .collect::<HashSet<&str>>()
        .into_iter()
        .filter(|name| facility_counts.get_count(name) > 0)
        .collect();
    facility_names.sort_unstable();

    // Wood Blocks/Mineral Sand produced as a side effect — purely informational (see doc comment
    // on `ProductionPlan::byproduct_rates`). A byproduct only ever comes from a raw item being
    // GROWN (every processed item's `byproduct` is always `None` — see the data loaders), so this
    // only ever applies to grower facilities, credited from `grower_assignment`'s exact plot
    // counts rather than a fractional rate: a plot assigned to grow something produces its full
    // byproduct yield regardless of whether the downstream recipe it feeds ends up using all of
    // what it grows (the residual-imprecision case noted in `final_rate`'s doc comment) — the
    // byproduct comes from growing, not from what happens to the harvest afterward. Kept as a rate
    // + lead time here (not yet multiplied by any duration, since no goal is known at this point)
    // — `time_to_reach_goal` turns these into totals once a plan's duration is known.
    let mut byproduct_rates: Vec<(String, f64, f64)> = Vec::new();
    let mut credit_byproduct = |item: &ProductionItem, fraction: f64| {
        let Some((ref resource, amount)) = item.byproduct else {
            return;
        };
        let facility_count = facility_counts.get_count(&item.facility) as f64;
        let rate = fraction * amount as f64 * facility_count / item.production_time;
        if rate <= 0.0 {
            return;
        }
        let lead = item_lead_time(&item.name, &item_map, 0);
        byproduct_rates.push((resource.clone(), rate, lead));
    };
    for ((facility, chain_name), &count) in &grower_assignment {
        if count == 0 {
            continue;
        }
        let Some(&eff) = eff_by_name.get(chain_name.as_str()) else {
            continue;
        };
        // A grower facility hosts exactly one raw item per chain (it only ever grows one crop),
        // so the first (only) hosted item there is the one actually being grown.
        let hosted_raw_item = eff
            .facility_demand
            .iter()
            .find(|(f, _, _)| f == facility)
            .and_then(|(_, _, hosted)| hosted.first());
        if let Some(raw_name) = hosted_raw_item {
            if let Some(&raw_item) = item_map.get(raw_name.as_str()) {
                let fraction = count as f64 / facility_counts.get_count(facility) as f64;
                credit_byproduct(raw_item, fraction);
            }
        }
    }

    let coin_items: Vec<PlanStep> = facility_names
        .iter()
        .flat_map(|&name| -> Vec<PlanStep> {
            if is_grower(name) {
                // Authoritative: `grower_assignment` was fixed BEFORE any rate capping, as the
                // whole-unit plot assignment everything else was derived from — not re-derived
                // here from a fractional view, so it stays exactly consistent with Total Time and
                // the Product Breakdown even in the rare case (see `final_rate`'s doc comment)
                // where a chain can't fully use every plot assigned to it due to a bottleneck at
                // one of its OTHER grower facilities.
                let total_owned = facility_counts.get_count(name);
                let mut assigned: Vec<(&str, &ProductionEfficiency, u32)> = grower_assignment
                    .iter()
                    .filter(|((facility, _), &count)| facility.as_str() == name && count > 0)
                    .filter_map(|((_, chain_name), &count)| {
                        eff_by_name.get(chain_name.as_str()).map(|&eff| (chain_name.as_str(), eff, count))
                    })
                    .collect();
                assigned.sort_by(|a, b| b.2.cmp(&a.2));

                if assigned.is_empty() {
                    return vec![PlanStep {
                        item_name: None,
                        facility: name.to_string(),
                        facility_count: total_owned,
                        status: PlanStepStatus::NothingAvailable,
                        reason: "No profitable item currently available".to_string(),
                    }];
                }

                // What's actually grown here for `eff`'s chain (`facility_demand`'s hosted item —
                // a grower only ever grows one crop, so there's exactly one), and how it's used.
                let describe = |eff: &ProductionEfficiency| -> (String, String) {
                    let hosted = eff
                        .facility_demand
                        .iter()
                        .find(|(f, _, _)| f == name)
                        .and_then(|(_, _, items)| items.first().cloned())
                        .unwrap_or_default();
                    let sentence = if eff.item.facility == name {
                        "Sells directly".to_string()
                    } else {
                        format!("Used for {}", eff.item.name)
                    };
                    (hosted, sentence)
                };

                let idle = total_owned.saturating_sub(assigned.iter().map(|(_, _, c)| c).sum());
                let mut steps: Vec<PlanStep> = assigned
                    .iter()
                    .map(|(_, eff, count)| {
                        let (label, reason) = describe(eff);
                        PlanStep {
                            item_name: Some(label),
                            facility: name.to_string(),
                            facility_count: *count,
                            status: PlanStepStatus::Producing,
                            reason,
                        }
                    })
                    .collect();
                if idle > 0 {
                    steps.push(PlanStep {
                        item_name: None,
                        facility: name.to_string(),
                        facility_count: idle,
                        status: PlanStepStatus::Idle,
                        reason: "No further profitable use found".to_string(),
                    });
                }
                return steps;
            }

            // Processor facility: a physically dedicated unit's achievable rate can only be >=
            // its share of a jointly-run one (its ceiling is a whole unit's worth of throughput,
            // not a fraction of it), so whole-unit dedication never changes any rate/total
            // computed above — it's a pure relabeling of which unit does what. The exclusion loop
            // in `find_production_plan` already guarantees no processor facility has more
            // contributors than owned units by this point (a processor can only ever be "set and
            // left" on one recipe — see `build_processor_usage`'s doc comment), so this always
            // resolves to whole-unit dedication, never a time-share percentage.
            let mut contributors: Vec<(&ProductionEfficiency, f64, f64)> =
                facility_usage.get(name).cloned().unwrap_or_default();
            contributors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if contributors.is_empty() {
                return vec![PlanStep {
                    item_name: None,
                    facility: name.to_string(),
                    facility_count: facility_counts.get_count(name),
                    status: PlanStepStatus::NothingAvailable,
                    reason: "No profitable item currently available".to_string(),
                }];
            }

            // What item `eff` produces here. For `eff`'s own root facility this is just its own
            // item; for any other facility touched by `eff`'s chain, look up which specific
            // item(s) it hosts there (`facility_demand`'s third element) rather than the whole
            // chain's name.
            let describe = |eff: &ProductionEfficiency| -> (String, String) {
                if eff.item.facility == name {
                    (eff.item.name.clone(), "Sells directly".to_string())
                } else {
                    let hosted = eff
                        .facility_demand
                        .iter()
                        .find(|(f, _, _)| f == name)
                        .map(|(_, _, items)| items.join("+"))
                        .unwrap_or_default();
                    (hosted, format!("Used for {}", eff.item.name))
                }
            };

            let owned = facility_counts.get_count(name);

            // A dedicated unit only ever needs to cover a contributor's own fractional demand, so
            // rounding UP to the next whole unit (never down) guarantees it's never under-supplied
            // relative to the shared-time version. Every contributor's rounded-up need is
            // guaranteed to fit within what's owned by this point (the exclusion loop in
            // `find_production_plan` already resolved any contention before this code runs), so
            // whole-unit dedication always applies; the leftover (if any) is genuinely idle
            // capacity, same concept as a grower's idle plots.
            let needed: Vec<u32> = contributors.iter().map(|(_, f, _)| f.ceil() as u32).collect();
            let total_needed: u32 = needed.iter().sum();
            debug_assert!(
                total_needed <= owned,
                "processor contention should have been resolved by find_production_plan's \
                 exclusion loop before coin_items is built"
            );

            let mut steps: Vec<PlanStep> = contributors
                .iter()
                .zip(&needed)
                .map(|((eff, _, _), &count)| {
                    let (label, reason) = describe(eff);
                    PlanStep {
                        item_name: Some(label),
                        facility: name.to_string(),
                        facility_count: count,
                        status: PlanStepStatus::Producing,
                        reason,
                    }
                })
                .collect();
            let idle = owned.saturating_sub(total_needed);
            if idle > 0 {
                steps.push(PlanStep {
                    item_name: None,
                    facility: name.to_string(),
                    facility_count: idle,
                    status: PlanStepStatus::Idle,
                    reason: "No further profitable use found".to_string(),
                });
            }
            steps
        })
        .collect();

    Some(ProductionPlan {
        currency: currency.to_string(),
        rate_per_second,
        income_streams,
        coin_items,
        byproduct_rates,
    })
}

/// Turns a [`ProductionPlan`] plus a goal amount into a concrete time-to-target, using the plan's
/// already-computed rates — no facility-allocation re-solve, so this is cheap enough to call on
/// every keystroke of a goal-amount input. Returns `None` only in the pathological case where the
/// binary search exceeds ~317 years without reaching `target` (treated as genuinely infeasible;
/// shouldn't happen for a `plan` returned by `find_production_plan`, whose `income_streams` always
/// has at least one item with positive `rate_per_second` when `Some` is returned).
pub fn time_to_reach_goal(plan: &ProductionPlan, target: f64, current: f64) -> Option<GoalResult> {
    let delta = (target - current).max(0.0);

    if delta <= 0.0 {
        return Some(GoalResult {
            total_time: 0.0,
            amount_produced: 0.0,
            products: vec![],
            byproducts: vec![],
        });
    }

    // Amount accumulated by time `t`: each income stream contributes nothing until its own lead
    // time has passed, then its steady-state rate after that. Monotonically non-decreasing in
    // `t`, so the minimal `t` reaching `delta` can be found by binary search.
    let amount_at = |t: f64| -> f64 {
        plan.income_streams
            .iter()
            .map(|p| p.rate_per_second * (t - p.lead_time).max(0.0))
            .sum()
    };

    let mut hi = 3600.0_f64;
    while amount_at(hi) < delta {
        hi *= 2.0;
        if hi > 1e10 {
            // ~317 years — treat as genuinely infeasible.
            return None;
        }
    }
    let mut lo = 0.0_f64;
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if amount_at(mid) >= delta {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let total_time = hi;
    let amount_produced = amount_at(total_time);

    // Fill in each income stream's actual contribution over the plan's duration, and drop any
    // that never got past their own lead time (they were claimed, but the target was reached
    // before they produced anything) — reported per-facility in `plan.coin_items`, but not worth
    // a row in this item-level breakdown. Sorted by total worth, most valuable first.
    let mut products: Vec<PlanProduct> = plan
        .income_streams
        .iter()
        .cloned()
        .map(|mut p| {
            let active_time = (total_time - p.lead_time).max(0.0);
            p.total_value = p.rate_per_second * active_time;
            p.total_units = p.units_per_second * active_time;
            p
        })
        .filter(|p| p.total_value > 0.0)
        .collect();
    products.sort_by(|a, b| {
        b.total_value
            .partial_cmp(&a.total_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut byproduct_totals: HashMap<&str, f64> = HashMap::new();
    for (resource, rate, lead) in &plan.byproduct_rates {
        let active = (total_time - lead).max(0.0);
        *byproduct_totals.entry(resource.as_str()).or_insert(0.0) += rate * active;
    }
    let mut byproducts: Vec<(String, f64)> = byproduct_totals
        .into_iter()
        .filter(|(_, amount)| *amount > 0.0)
        .map(|(resource, amount)| (resource.to_string(), amount))
        .collect();
    byproducts.sort_by(|a, b| a.0.cmp(&b.0));

    Some(GoalResult {
        total_time,
        amount_produced,
        products,
        byproducts,
    })
}
