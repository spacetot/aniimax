# New Beta — Changes Tracking

## Implementation status (this section updated as code changes land)

**Implemented (Farmland, Woodland, Mineral Pile — all confirmed data as of this pass):**
- `data/farmland.csv`, `data/woodland.csv`, `data/mineral_pile.csv` rewritten with new items/values.
- `ProductionItem` gained two new fields: `workload: Option<f64>` (informational) and
  `byproduct: Option<(String, u32)>` (Wood Blocks / Mineral Sand, informational — not wired
  into profit/currency calculations, since these are progression resources).
- `MineralRow` now carries `workload` instead of `production_time`; the loader (`data.rs` and
  `wasm.rs`) derives `production_time = workload / MINERAL_PILE_WORKLOAD_RATE`, where
  `MINERAL_PILE_WORKLOAD_RATE` (in `models.rs`) is **300.0 / 209.0 ≈ 1.4354** workload/sec,
  calibrated from the single confirmed Shell data point. **This is applied uniformly to every
  Mineral Pile item (Clay, Quartz, Gem included) — an unverified assumption.** If a second
  calibration point on a different Mineral Pile item ever comes in, revisit this constant (it
  may need to be per-item rather than facility-wide).
- Verified end-to-end in the browser: module gating (Ecological Module → Quick Wheat, Mineral
  Detector → Quick Shell/Quick Quartz), facility-level gating, and profit/time ranking all work
  correctly with the new data. Confirmed old Aniipod Maker recipes referencing now-removed
  Mineral Pile items (`rock`, `copper`) are silently excluded rather than crashing — expected,
  since Aniipod Maker hasn't been updated for the new beta yet.

**Assumptions made during implementation that need verification:**
- **Quick Rose module requirement**: guessed `ecological_module:1` (same as confirmed Quick
  Wheat) — not actually confirmed for Quick Rose specifically.
- **Quick Lemon / Quick Coconut module requirement**: guessed `ecological_module:1` — not
  confirmed at all (we don't know if Woodland Quick items even use the Ecological Module, vs.
  some other gate).
- **Woodland sell currency**: assumed all-coins for every item (matching what's confirmed for
  Mineral Pile). Old data had several Woodland items selling for coupons — unconfirmed whether
  that's still true for any of Bamboo/Rubber/Pine/Willow in the new beta.
- **Farmland/Woodland `requires_fertilizer` thresholds** (Farmland level≥4, Woodland level≥3)
  left unchanged from old data — fertilizer mechanics haven't been re-confirmed for the new beta
  at all (open question #6).

**Also implemented: Nimbus Bed** (workload-based like Mineral Pile, own calibration constant
`NIMBUS_BED_WORKLOAD_RATE` in `models.rs` derived from the Petals data point; `fertilizer` item
removed entirely from data + CLI doc comment since it no longer exists in-game).

**Update (later sessions): all currency-producing facilities are now implemented** — Jukebox
Dryer was updated with real data (section 17), and Dance Pad Polisher/Aniipod Maker were removed
entirely (section 20) since they don't produce coins/coupons/Bud Tickets. Only the 6 Auxiliary
Facilities remain unimplemented (deferred, not item-producers). The growing-environment
mechanic (warm/cold/etc.) also isn't modeled at all yet (not accessible to user). Electric Mode
is deferred everywhere per the agreed
sequencing plan (section 13) — Normal Mode first, electric mode as a follow-up pass.

**Architecture note — new facilities:** the 11 brand-new facilities (Grass Blossom Mat,
Starfall Hammock, Tidewhisper Sandcastle, Dewy House, Phonolfactory Table, Bouncy Brew Keg,
Claw Game Cooker, Joy Wheel Loom, Storage Unit, Crackle Power Pole/Generator, Heat Furnace,
Cooling Unit, Sunlamp) are NOT yet wired into `FacilityCounts`/`JsFacilityConfig`/the web UI.
Adding each one the way the original 9 facilities work requires touching ~6 places per facility
(struct field + match arm in `models.rs`, JS config + HTML markup in `web/`, construction sites
in `wasm.rs`). With 11 more facilities incoming, doing that per-facility will be a lot of
repetitive, error-prone edits. Worth considering a refactor to a generic
`HashMap<String, FacilityConfig>` before wiring in the new ones, rather than bolting each one on
individually — but holding off on that decision until more data is in and it's clearer which of
these are even "producible item" facilities vs. auxiliary/passive ones (Storage/Power/Climate
likely aren't ProductionItem sources at all).

**Correction applied (this pass):** Quick Lemon and Quick Coconut module guesses were wrong —
fixed from `ecological_module:1` (guess) to the now-confirmed `ecological_module:2` and
`ecological_module:3` respectively. Quick Rose fixed from guessed `ecological_module:1` to
confirmed `ecological_module:4`. Quick Wheat's guess (`ecological_module:1`) was already
correct. Quick Shell (`mineral_detector:1`) and Quick Quartz (`mineral_detector:3`) were
already correct per the full module table below.

### 10. Full Module/Component system (user-provided, comprehensive)

The Homeland is organized into named areas containing "components" (modules), each with 4-5
levels gating specific unlocks. Full list as provided by user:

**RV Cabin area:**

| Module | Lvl 1 | Lvl 2 | Lvl 3 | Lvl 4 |
|---|---|---|---|---|
| Ecological Module | Quick Wheat | Quick Lemon | Quick Coconut | Quick Rose |
| Kitchen Module | Unlocks Aniimo feeding | Unlocks Premium Flower Bread (Claw Game Cooker) | Unlocks Premium Soy Sauce Tofu (Claw Game Cooker) | Unlocks Premium Potato Kvass (Bouncy Brew Keg) |
| Mineral Detector | Quick Shell | Quick Scales (Grass Blossom Mat) | Quick Quartz (Mineral Pile) | Quick Pearl (Tidewhisper Sandcastle) |
| Crafting Module | Premium Wood Sculpture (Crafting Table) | Premium Wind Chime (Crafting Table) | Premium Woven Toy (Crafting Table) | Premium Dream Catcher (Crafting Table) |

**Rooftop Loft area:**

| Module | Lvl 1 | Lvl 2 | Lvl 3 | Lvl 4 | Lvl 5 |
|---|---|---|---|---|---|
| Power Module | Unlocks Crackle Generator + Crackle Power Pole | Generator Lvl 2 | Generator Lvl 3 | Generator Lvl 4 | Generator Lvl 5 |
| Plant Research Module | Unlocks Colorful Mutation; ↑ Huge Mutation probability | Unlocks Sparkling Mutation; ↑ Colorful Mutation probability | ↑ probability of all mutations | ↑ probability of all mutations | — |

**Ecological Module — now fully mapped and corrected in data:**
- Lvl 1 → Quick Wheat (Farmland) ✅ already correct
- Lvl 2 → Quick Lemon (Woodland) — **corrected from wrong Lvl 1 guess**
- Lvl 3 → Quick Coconut (Woodland) — **corrected from wrong Lvl 1 guess**
- Lvl 4 → Quick Rose (Farmland) — **corrected from wrong Lvl 1 guess**

**Mineral Detector — fully mapped, confirms existing data was right, reveals 2 new facilities:**
- Lvl 1 → Quick Shell (Mineral Pile) ✅ already correct
- Lvl 2 → Quick Scales — for a facility called **Grass Blossom Mat** (NEW, not yet catalogued)
- Lvl 3 → Quick Quartz (Mineral Pile) ✅ already correct
- Lvl 4 → Quick Pearl — for a facility called **Tidewhisper Sandcastle** (NEW, not yet catalogued)

**Kitchen Module — reveals 2 new facilities (or new names for existing ones — unconfirmed):**
- Lvl 1 → unlocks Aniimo feeding (a mechanic, not a craftable item — likely related to the
  Energy system, e.g. feeding Aniimo directly instead of/alongside facility energy consumption)
- Lvl 2 → Premium Flower Bread, at **Claw Game Cooker** (NEW facility name — could be a rename
  of Carousel Mill or Jukebox Dryer, or genuinely new; old Kitchen Module gated
  `super_wheatmeal` at Carousel Mill, so there may be a naming/identity link worth checking)
- Lvl 3 → Premium Soy Sauce Tofu, also at Claw Game Cooker
- Lvl 4 → Premium Potato Kvass, at **Bouncy Brew Keg** (NEW facility name)

**Crafting Module — all 4 levels now gate Crafting Table premium recipes (old data only had 1):**
- Lvl 1 → Premium Wood Sculpture (old data: `advanced_wood_sculpture` at `crafting_module:1` —
  likely the same item, renamed "Premium" instead of "Advanced")
- Lvl 2 → Premium Wind Chime (NEW recipe, not in old data)
- Lvl 3 → Premium Woven Toy (NEW recipe, not in old data)
- Lvl 4 → Premium Dream Catcher (NEW recipe, not in old data)

**Power Module — entirely new mechanic, a dedicated power-generation building:**
- Unlocks **Crackle Generator** and **Crackle Power Pole** facilities at Lvl 1, with the
  Generator itself having levels 2-5 unlocked by further Power Module levels.
- This looks like a genuinely new way to produce Energy (a building that generates it passively
  or actively) rather than only consuming farmed/gathered items for energy, as in the old model
  and current beta data so far. Could significantly change energy-cost optimization once
  understood — worth prioritizing once accessible.

**Plant Research Module — entirely new mechanic, crop mutation system:**
- Unlocks probabilistic "mutations" for crops (Colorful, Sparkling, Huge, etc.) that presumably
  produce higher-value variants of existing Farmland/Woodland items. Not modeled at all yet —
  need to understand: does a mutation replace the normal yield, add bonus items, and what do
  mutated items sell for? This could meaningfully shift which crops are "best" once factored in.

**New facilities now known to exist (not yet catalogued in data):**
- Grass Blossom Mat (produces Scales, gated behind Mineral Detector Lvl 2)
- Tidewhisper Sandcastle (produces Pearl, gated behind Mineral Detector Lvl 4)
- Claw Game Cooker (produces Premium Flower Bread, Premium Soy Sauce Tofu — possibly a
  rename/replacement for Carousel Mill or Jukebox Dryer, needs investigation)
- Bouncy Brew Keg (produces Premium Potato Kvass)
- Crackle Generator + Crackle Power Pole (power/energy generation, not item production)

Also confirms the Homeland is organized into distinct **areas** (RV Cabin, Rooftop Loft, and
presumably others) each containing several component/module slots — useful organizing context
but doesn't by itself require a data model change.


Working notes on what changed vs. the old (current repo) data/mechanics, gathered while
the user still has partial access to the new beta. Update this file as new info comes in;
treat it as the source of truth for the eventual data/code rewrite.

## Confirmed mechanic changes

### 1. Workload / Efficiency replaces flat production_time (for some facilities)

- Some facilities still show a **flat, fixed Time** regardless of Aniimo dispatched
  (raw-gathering facilities so far: Farmland, Mineral Pile — the *Time* shown is fixed,
  workload doesn't apply to the timer itself).
- Other facilities (so far confirmed: Nimbus Bed) show a **Workload** stat instead, and the
  actual countdown depends on which Aniimo is dispatched and their **Efficiency %**.
- Working hypothesis (2 calibration points so far):
  - `effective_time = base_time_at_100%_efficiency / efficiency_fraction`
  - `workload` itself does NOT convert to time via one universal constant — the
    workload:time ratio differs per item/facility (Shell ≈1.44 workload/sec at 100%,
    Petals ≈3.11 workload/sec at 100% implied). So workload is likely a secondary/flavor
    stat (maybe Aniimo XP or facility progress), not the direct time driver.
  - What the optimizer actually needs is `base_time_at_100%_efficiency` (functionally
    the same role as the old `production_time` field) + a per-facility (or per-task)
    `efficiency%` input from the user, analogous to how facility level worked before.

  **Calibration data points:**
  | Item | Facility | Workload | Efficiency | Observed Time | Implied base_time @100% |
  |------|----------|----------|------------|----------------|--------------------------|
  | Shell | Mineral Pile (L1) | 300 | 100% | 3m29s = 209s | 209s (direct) |
  | Petals | Nimbus Bed | 2250 | 140% | 8m37s = 517s | 723.8s (derived) |

  **Still need:** more data points, ideally 2 different efficiency % on the *same* item,
  to confirm the `time = base_time / efficiency_fraction` relationship is actually linear
  (not just assumed from 2 unrelated items).

- The Efficiency % itself appears to depend on which Aniimo is dispatched (species match
  to facility, Aniimo level vs. the facility's "Recommended" level badge, possibly gear/buffs).
  UI shows a "Recommended: [icon] L#" badge per facility screen. Not yet understood in detail
  — user will need to supply more examples once they have multiple Aniimo to compare.

### 2. Planting cost changed: seed bags (bought with gold), not a direct per-plant coin cost

- Old model: Farmland items had a flat `cost` in coins to plant directly (e.g. potatoes cost
  2 coins).
- New beta: Farmland items consume `1x seed bag` per planting action (shown as owned/required,
  e.g. `13063/1`). **Confirmed: seed bags are bought with gold (coins)**, at a per-crop price
  (e.g. Potato seed = 2 gold, Sugarcane seed = 3 gold, Rice seed = 3 gold, Cotton seed = 9 gold).
  So functionally this is close to the old model (coin cost per planting), just mediated through
  a seed-bag item rather than a direct coin deduction. For the optimizer's purposes this can
  likely be modeled the same way as the old flat `cost` field (coins spent per batch), unless
  seed bags turn out to be obtainable another way too (crafting, rewards) which would make them
  cheaper than buying — worth double-checking later once more is known.

### 3. Crown-badged items sell for a different currency

- Confirmed: non-crown items sell for **coins** (same as old model). Crown-badged items sell
  for **tickets**.
- **Open question:** is the full currency set now {coins, tickets} (coupons renamed/removed),
  or {coins, coupons, tickets} (three currencies)? User was unsure — pending confirmation.
- Old model also had `coupons` as a sell currency for some Woodland/Mineral Pile/Crafting/Dance
  Pad Polisher items. Need to check whether those specific items still exist and what currency
  they sell for now.

## Data observed so far (raw values from screenshots)

- **Potato** (Farmland, facility level 2 shown): input `1x seed bag` (owned 0) → yields `6x potato`,
  Time `7m 30s` (450s, flat — not workload-based), Provides Energy `380`.
- **Shell** (Mineral Pile, facility level 1): Workload `300`, at 100% efficiency → `3m 29s` (209s).
- **Petals** (Nimbus Bed): Workload `2250`, at 140% efficiency → `8m 37s` (517s).
- **Shell Ornament** (Crafting Table, level 1 formula): recipe `12x shell` (686 owned at time of
  screenshot) → 1x Shell Ornament. Workload `18` (not a time). "Home materials" category —
  crafting facilities now show tiered "Formula" unlocks per facility level (Level 1..5+ seen
  in the product-select screen), consistent with old `facility_level` gating, but now
  presented as a picker UI with multiple item icons per tier and some crown-badged (ticket
  currency) variants mixed into the same tier.

### 4. "Quick Formula" alternate recipes

- Product-select screens show a second recipe option per base item, badged with a green
  up-arrow icon, labeled "Quick Formula: <item>" (e.g. "Quick Formula: Wheat"). Sits next to
  the plain item icon within the same facility-level tier. Likely plays the role the old
  `high_speed_*` module-gated variants used to play, but doesn't require an obvious module —
  unlock condition not yet confirmed (could still be module-gated, or could be a different
  unlock like recipe unlock via research/currency).
- Also seen at Level 4 tier on a *locked* rose-like icon, so Quick Formula variants exist at
  higher tiers too, not just Level 1.
- **Quick Formula: Wheat** data (Farmland, Level 1): cost `1x seed bag` (owned 13065 at time of
  screenshot), yield `8`, Time `2m 30s` (150s, flat), Provides Energy `300`.
  - Notably *slower and lower-yield* than the old plain `wheat` (90s / yield 10 / energy 809),
    so "Quick Formula" is not simply a strict upgrade — need the plain/regular Wheat card
    for the same tier to compare directly and figure out the actual tradeoff.
- **Regular Wheat** data (Farmland, Level 1): cost `1x seed bag`, yield `5`, Time `2m 30s`
  (150s, flat), Provides Energy `300`.
- **Direct comparison confirmed:** Quick Formula and Regular Wheat have identical cost (1 seed
  bag), time (150s), and energy (300) — the ONLY difference is yield (8 vs 5, +60%). So
  "Quick Formula" plays the same role the old `high_speed_wheat` module-gated variant played:
  same time/cost, better yield. Likely a simple relabel of the same mechanic, not a new one.
  - **Confirmed:** Quick Formula: Wheat requires **Ecological Module Lvl 1** — identical
    unlock condition to the old `high_speed_wheat` (`ecological_module:1`). Strong signal
    that the module system carried over unchanged; "Quick Formula: X" is very likely just
    the new UI label for what used to be `high_speed_X` in the data files.

### 5. Farmland Level 2 items (rebalanced, not just relabeled)

| Item | Cost (gold/seed) | Time | Yield | Level | Sell value | Energy |
|---|---|---|---|---|---|---|
| Potatoes | 2 | 450s (7m30s) | 6 | 2 | 5 | 380 |
| Sugarcane | 3 | 750s (12m30s) | 5 | 2 | 9 | 760 |
| Rice | 3 | 750s (12m30s) | 10 | 2 | 5 | 380 |
| Cotton | 9 | 2250s (37m30s) | 6 | 2 | 23 | 0 |

Comparison vs old data:
- Potatoes: cost & time identical to old (2 gold, 450s), yield doubled (3→6), level dropped
  1→2, but sell value dropped hard (17→5) and energy dropped hard (5390→380). Net profit/batch
  actually *lower* than old despite the yield doubling (old: 17×3-2=49; new: 5×6-2=28).
- Sugarcane: cheaper (3 vs 8), much faster (750s vs 1350s), yield same (5), sell value dropped
  (32→9), energy dropped (6800→760).
- Rice: cheaper (3 vs 5), faster (750s vs 810s), yield same (10), sell value dropped (10→5),
  energy dropped (3690→380).
- Cotton: cheaper (9 vs 20), faster (2250s vs 2700s), yield same (6), level dropped 3→2, sell
  value dropped hard (67→23), energy went from unlisted (—) to explicitly 0 (can no longer be
  consumed for energy).

Confirms a full rebalance: costs/times generally went down, but so did sell values and energy
— this isn't a simple buff, ratios shifted meaningfully and need fresh optimizer runs once the
full dataset is in, not just old assumptions about which crop is "best."

### 6. Full Farmland table (user-compiled, all levels) — as of this message

| Item | Level | Time | Yield | Sell value | Energy | Seed cost (gold) |
|---|---|---|---|---|---|---|
| Wheat | 1 | 150s (2m30s) | 5 | 1 | 300 | 0 |
| Quick Wheat | 1 | 150s (2m30s) | 8 | 1 | 300 | 0 (assumed same as Wheat) |
| Potatoes | 2 | 450s (7m30s) | 6 | 5 | 380 | 2 |
| Sugarcane | 2 | 750s (12m30s) | 5 | 9 | 760 | 3 |
| Rice | 2 | 750s (12m30s) | 10 | 5 | 380 | 3 |
| Cotton | 2 | 2250s (37m30s) | 6 | 23 | 0 | 9 |
| Strawberries | 3 | 2250s (37m30s) | 5 | 72 | 2090 | 24 |
| Soybeans | 3 | 2250s (37m30s) | 8 | 45 | 1300 | 24 |
| Lavender | 4 | 2250s (37m30s) | 5 | 189 | 0 | 63 |
| Agave | 4 | 2250s (37m30s) | 4 | 236 | 0 | 63 |
| Rose | 4 | 2250s (37m30s) | 6 | 158 | 0 | 63 |
| Quick Rose | 4 | 2250s (37m30s) | 9 | 158 | 0 | 63 (assumed same as Rose) |
| Grapes | 5 | 2250s (37m30s) | 5 | 306 | 2510 | 102 |
| Ginseng | 5 | 2250s (37m30s) | 2 | 765 | 6270 | 102 |
| Pumpkin | 5 | 4500s (1h15m) | 1 | 1530 | 12540 | 170 |

Note: rows for Potatoes/Sugarcane/Rice/Cotton cross-checked against the earlier individually-
reported values and match exactly — data source looks reliable/consistent. Seed costs from
user's consolidated seed table, matched by plant name.

**Farmland data now considered essentially complete**, except:
- Quick Wheat / Quick Rose seed cost assumed same as base crop (not explicitly confirmed, but
  consistent with how seed bags worked for regular Wheat).
- Module unlock requirement for Quick Rose not yet confirmed (only Quick Wheat →
  Ecological Module Lvl 1 confirmed so far).
- Growing environment mechanic (see section 8) still unknown/not accessible.

### 7. Woodland now has dual output: Product + Wood Blocks

Full user-compiled table, with seed costs now filled in from the consolidated seed table
(matched by tree name — Chestnut Tree, Willow, Bamboo, Lemon Tree, Palm Tree, Rubber Tree,
Coconut Tree, Maple Tree, Walnut Tree, Pine Tree):

| Item | Level | Time | Product Yield | Wood Blocks Yield | Value | Seed cost (gold) |
|---|---|---|---|---|---|---|
| Willow Wood | 1 | 375s (6m15s) | 2 | 1 | 8 | 1 |
| Chestnut | 2 | 1125s (18m45s) | 3 | 8 | 38 | 8 |
| Bamboo | 2 | 2250s (37m30s) | 4 | 15 | 56 | 15 |
| Lemon | 2 | 2250s (37m30s) | 5 | 15 | 45 | 15 |
| Quick Lemon | 2 | 2250s (37m30s) | 8 | 15 | 45 | 15 (assumed same as Lemon) |
| Palm Bark | 3 | 2250s (37m30s) | 4 | 39 | 146 | 39 |
| Coconut | 3 | 2250s (37m30s) | 5 | 39 | 117 | 39 |
| Maple Syrup | 3 | 2250s (37m30s) | 6 | 39 | 98 | 39 |
| Quick Coconut | 3 | 2250s (37m30s) | 8 | 39 | 117 | 39 (assumed same as Coconut) |
| Natural Rubber | 4 | 2250s (37m30s) | 2 | 102 | 765 | 102 |
| Walnut | 4 | 2250s (37m30s) | 3 | 102 | 510 | 102 |
| Pine Tree Hardwood | 4 | 2250s (37m30s) | 1 | 102 | 1430 | 102 |

(Lemon/Quick Lemon value corrected to 45/45 per user follow-up — see below.)

**Confirmed — Wood Blocks purpose:** primarily used to upgrade Homeland/RV level (a
progression resource, not directly a production input). Mineral Pile has an equivalent
byproduct: **Mineral Sand** (also for Homeland upgrades).

**Confirmed — Resource Exchange shop rates** (each line capped at 10 exchanges/day):
| Exchange | Rate | Implied unit price |
|---|---|---|
| Wood Blocks → Home Coin | 200 → 1000 | 5 coins/Wood Block (sell) |
| Mineral Sand → Home Coin | 200 → 1000 | 5 coins/Mineral Sand (sell) |
| Home Coin → Wood Block | 2000 → 100 | 20 coins/Wood Block (buy) |
| Home Coin → Mineral Sand | 2000 → 100 | 20 coins/Mineral Sand (buy) |
| Mineral Sand → Wood Block | 200 → 100 | 2:1 (worse than routing via coins) |
| Wood Blocks → Mineral Sand | 200 → 100 | 2:1 (worse than routing via coins) |

Sell rate (5 coins/unit) gives a real coin-equivalent value for these byproducts, capped at
2000 units/day (10 × 200) per resource → max 10,000 coins/day per resource from this exchange.
Cross-exchanges are a bad deal vs. selling to coins then buying the other resource.

**Modeling decision:** treat Wood Blocks/Mineral Sand primarily as informational secondary
output (most players want them for Homeland upgrades), but the optimizer could offer an
*optional* "value byproducts at exchange-sell-rate" mode using the 5 coins/unit rate, capped
by the daily limit, for players who want to factor that in.

**Confirmed — Energy still applies to Woodland.** New values (compare to old in parens):
- Chestnut: 1750 (old: 10560)
- Lemon: 2100 (old: 17270) — note: user later corrected Lemon's *value* field, not energy;
  energy for Lemon stands at 2100 pending Quick Lemon energy confirmation (assume same as
  Lemon per the value-parity pattern, but not yet explicitly confirmed)
- Coconut: 2160 (old: 17960)
- Maple Syrup: 1800 (old: 32060)
- Walnut: 4180 (old: 28790)
All big reductions vs. old data — consistent with the broader rebalance pattern seen elsewhere
(lower energy, lower sell values across the board).
Still open: energy for Willow Wood, Bamboo, Palm Bark, Natural Rubber, Pine Tree Hardwood
(old data had none for these — willow/bamboo/rubber/pine were NULL — so possibly still N/A,
but worth confirming since palm_bark is new/renamed from old "palm").

**Resolved — Quick Lemon value:** user corrected earlier report — **both Lemon and Quick
Lemon are actually value 45** (not 98/117 as first reported). This restores the expected
pattern: Quick variants match base item value, only yield differs.

**Cost/seed price:** deferred by user, to be gathered together with Farmland seed costs
(Wheat, Strawberries, Soybeans, Lavender, Agave, Rose, Grapes, Ginseng, Pumpkin) in a later
pass.

### 8. New mechanic: growing environments (warm/cold/etc.)

Confirmed **entirely new** — does not exist anywhere in the old model (no environment/climate
concept in the old CSVs or Rust structs at all). Some crops apparently require a specific
environment (warm, cold, etc.) to grow.

**Open question — how restrictive is this?**
- If **per-plot** (a plot must be set to one environment and can only grow matching crops
  until changed), this breaks the optimizer's core assumption that N identical facility slots
  can produce anything interchangeably — would need to model environment as a constraint on
  facility allocation, similar to how facility levels gate items today.
- If **automatic/cosmetic** (game just displays it, doesn't restrict planting), no model
  change needed beyond maybe a display label.
- Not yet confirmed which — pending user's further play.

### 9. Mineral Pile — workload-based, no seed cost, Aniimo skill-driven

User-compiled table:

| Item | Level | Workload | Product Yield | Mineral Sand Yield | Value |
|---|---|---|---|---|---|
| Shell | 1 | 300 | 2 | 2 | 11 |
| Quick Shell | 1 | 300 | 3 | 3 | 11 |
| Clay | 2 | 2250 | 10 | 24 | 34 |
| Quartz | 3 | 2700 | 63 | 4 | 221 |
| Quick Quartz | 3 | 2700 | 63 | 6 | 221 |
| Gem | 4 | 2700 | 102 | 2 | 714 |

**Confirmed:** Mineral Pile items have **no seed/planting cost** — instead, "an Aniimo with a
varying skill level would get the task." This is a clean confirmation of the workload/efficiency
hypothesis in section 1: no coin/item cost to start, completion speed purely driven by which
Aniimo (and their skill) is dispatched. Also confirms the Shell calibration data point from
earlier (workload 300, 100% efficiency → 209s) lines up exactly with this table's Shell entry.

Quick variants don't follow one consistent transformation rule here (unlike Farmland/Woodland
where Quick = yield-only boost, same value/cost/time): Quick Shell boosts *both* Product Yield
(2→3) and Mineral Sand Yield (2→3) at the same workload/value; Quick Quartz keeps Product Yield
the same (63=63) but boosts Mineral Sand Yield (4→6). So each item's Quick variant needs to be
recorded individually, no shortcut assumption.

**Resolved:**
- Confirmed "Mineral Sand" here is the same resource as the Resource Exchange currency in
  section 7 (earlier "Mineral Dust" label was a typo, now corrected throughout).
- Sell currency: all coins — none of these sell for the crown/ticket currency, now known to be
  called **Bud Tickets** specifically.
- Module unlock: **Mineral Detector Lvl 1** for Quick Shell, **Mineral Detector Lvl 3** for
  Quick Quartz — same `mineral_detector` module concept as the old data, but required level
  differs per item (not a single flat unlock for everything).

### 11. Complete facility list (user-provided) — 20 facilities across 3 categories

Far larger scope than the original 9 facilities. Three categories:

**Materials (raw gathering, 8 total):** Farmland, Woodland, Mineral Pile, Nimbus Bed,
Grass Blossom Mat, Starfall Hammock, Tidewhisper Sandcastle, Dewy House

**Materials Processing (7 total):** Carousel Mill, Phonolfactory Table, Bouncy Brew Keg,
Crafting Table, Claw Game Cooker, Joy Wheel Loom, Jukebox Dryer

**Auxiliary Facilities (6 total, NOT direct item producers):** Storage Unit, Crackle Power Pole,
Crackle Generator, Heat Furnace, Cooling Unit, Sunlamp

**Key hypothesis:** Heat Furnace / Cooling Unit / Sunlamp are very likely the mechanism behind
the growing-environment system (section 8) — i.e. you build these Auxiliary Facilities to create
warm/cold/sunny conditions, rather than each plot having an environment dropdown. If true, this
reframes the environment constraint as "do you own the right climate-control building," which is
much simpler to model (a boolean unlock per environment type) than a per-plot allocation puzzle.
**Not yet confirmed** — need to check in-game once accessible.

Brand new facilities with **zero data yet**: Starfall Hammock, Dewy House, Phonolfactory Table,
Joy Wheel Loom, Storage Unit. (Grass Blossom Mat and Tidewhisper Sandcastle have partial info —
Scales/Pearl products — from the Mineral Detector module table in section 10; Bouncy Brew Keg
and Claw Game Cooker similarly have partial info from the Kitchen Module table.)

Auxiliary Facilities (Storage/Power/Climate) likely don't belong in the optimizer as
producible "items" at all — Storage Unit probably just raises inventory caps, Power facilities
generate/distribute energy passively, Climate facilities enable growing conditions. These are
probably better modeled as unlock flags / capacity numbers than as `ProductionItem` rows. Data
model implications TBD once we understand exactly how each one works in-game.

### 12. Nimbus Bed and Grass Blossom Mat — Aniimo Family requirement + Environment column

New mechanic: some facilities require a **specific Aniimo Family** (species), not just any
Aniimo with a skill level — e.g. Nimbi family for Wool, Iris family for Petals, Flutternym
family for Scales. This is likely what the "Recommended: [icon] L#" badges seen earlier were
partly indicating (family icon + level).

Also a new **Environment** column per item (separate from, or related to, the growing
environment mechanic in section 8) — most items show "N/A" but Quick Scales shows "Adequate".
Meaning unconfirmed — could be the same warm/cold/etc. system, or a distinct
condition/upkeep stat. **Open question**, pending user clarification.

User-provided tables (sell value not yet included — pending):

**Nimbus Bed:**
| Item | Aniimo Family | Workload | Output | Environment | Sell value |
|---|---|---|---|---|---|
| Wool | Nimbi | 2250 | 4 | N/A | 53 |
| Petals | Iris | 2250 | 6 | N/A | 35 |

Note: Petals workload (2250) and output (6) match the earlier calibration screenshot exactly —
consistent. Old data had a third Nimbus Bed item, `fertilizer`, not present in this table.

**Resolved: fertilizer no longer exists in the new beta at all.** This has real code
implications — the old model has a whole fertilizer subsystem: `ProductionItem.requires_fertilizer`
(gated by facility level thresholds — Farmland≥4, Woodland≥3), and
`ProductionEfficiency.fertilizer_per_batch` in the optimizer (added recently per git log: "add
fertilizer tracking to production efficiency calculations"). Since fertilizer is gone, this
entire subsystem is now dead weight and should be removed during the eventual full rewrite,
not just left as unused fields. Not urgent to rip out immediately (doesn't break anything to
leave dormant), but flagging so it doesn't get reimplemented for the new data by mistake.

**Grass Blossom Mat** (brand new facility, first data):
| Item | Aniimo Family | Workload | Output | Environment | Sell value |
|---|---|---|---|---|---|
| Scales | Flutternym | 2250 | 12 | N/A | 28 |
| Quick Scales | Flutternym | 2250 | 18 | Adequate | 28 |

Quick Scales module requirement already known from section 10: Mineral Detector Lvl 2. Sell
value confirmed same for both (28) — consistent with the established Quick-item pattern
(yield-only boost, same per-unit price). "Adequate" environment requirement meaning still
unknown — user will investigate once accessible in-game.
Unconfirmed: facility level, whether Grass Blossom Mat also yields a Mineral-Sand-style
byproduct. **Not yet wired into code** — this is a brand new facility, not yet added to
`FacilityCounts`/UI (see "Facility architecture" note below).

### 13. New mechanic: Manual mode vs. Electric mode

**Confirmed NOT modeled yet.** Many facilities apparently offer two operating modes:
- **Manual mode**: workload-based (Aniimo dispatch + efficiency%, as currently modeled for
  Mineral Pile/Nimbus Bed).
- **Electric mode**: consumes electricity instead, and reports a flat **duration** instead of a
  workload stat (closer to the old flat-`production_time` model, but resource-gated by
  electricity rather than free).

This likely connects to the Power Module / Crackle Generator / Crackle Power Pole facilities
(sections 10-11) — electricity is presumably generated there and spent to run electric-mode
production as an alternative to Aniimo labor.

**Not yet understood:**
- Whether electric mode changes yield/sell-value, or purely swaps the time/cost mechanism for
  the same output.
- Electricity source/generation rate/cost (only Crackle Generator, or others too?).
- Whether every item supports both modes, or only specific ones.
- How this should be modeled in the optimizer — likely need per-item `electric_duration` and
  `electricity_cost_per_batch` fields, plus logic to pick (or let the user pick) between manual
  and electric paths depending on which is faster/cheaper given available electricity.

This is a bigger data-model change than anything implemented so far (first mechanic requiring
the optimizer to choose between two distinct paths for the same item) — deferred until more
data comes in on how it actually works.

**Data points (Crafting Table):**
| Item | Workload (manual) | Electric mode duration |
|---|---|---|
| Shell Ornament | 18 | 18s |
| Pottery | 45 | 36s |

Initial hypothesis from Shell Ornament alone (workload = electric seconds, 1:1) is **rejected**
— Pottery's ratio is 36/45 = 0.8, not 1.0. No clean universal (or even per-facility) formula
relating workload to electric-mode duration is apparent from just these two points.
**Conclusion: treat electric-mode duration as its own independently-recorded stat per item**,
not something derived from workload — same treatment as manual-mode's `production_time` before
we started deriving it from workload for Mineral Pile/Nimbus Bed. Electricity cost per batch
still unknown for both items — needed before electric mode can be modeled as a real alternative
production path in the optimizer.

**Agreed sequencing plan:** gather and implement all facilities' Normal/Manual Mode (workload)
data first, across the full 20-facility list. Electric mode is a separate, dedicated pass to
tackle afterward once Normal Mode is complete everywhere.

Also confirms our earlier Shell/Petals calibration screenshots were genuinely Manual mode
(showed an Aniimo dispatched with an Efficiency% badge), so [[MINERAL_PILE_WORKLOAD_RATE]] and
[[NIMBUS_BED_WORKLOAD_RATE]] remain valid as manual-mode-only calibrations, not mixed up with
electric-mode data.

### 14. Carousel Mill and Crafting Table — Normal Mode data implemented

**Design pivot: unified workload rate constant.** Previously used separate per-facility
calibration constants (`MINERAL_PILE_WORKLOAD_RATE`, `NIMBUS_BED_WORKLOAD_RATE`). Now that
identical workload "tier" values are showing up across unrelated facilities (Wheatmeal at
Carousel Mill and Shell Ornament at Crafting Table both have workload 18; more tiers line up
at 23, 27, 45, 54, 81, 108, 162), consolidated into one `WORKLOAD_RATE_ESTIMATE` constant
(`models.rs`), based on the Shell/Mineral-Pile data point (our only *direct* 100%-efficiency
measurement — 300/209 ≈ 1.4354 workload/sec), applied universally across all workload-based
facilities. Still provisional/estimated, but more defensible than maintaining multiple
unconfirmed facility-specific guesses.

**Carousel Mill — implemented (Normal Mode only; Electric Mode duration data recorded here but
NOT yet wired into code per the agreed sequencing plan):**

| Item | Level | Ingredients | Workload | E-duration | Value | Energy |
|---|---|---|---|---|---|---|
| Wheatmeal | 1 | 30 Wheat | 18 | 18s | 60 | 12340 |
| Rice (product) | 2 | 30 Rice | 23 | 18s | 225 | 12600 |
| Tofu | 3 | 8 Soybean | 23 | 18s | 555 | 12960 |
| Coconut Oil | 3 | 5 Coconut | 23 | 18s | 780 | 12960 |

Old `super_wheatmeal` (kitchen_module:2 gated) dropped — not present in the new table, and its
old gating is now contradicted by the confirmed Kitchen Module mapping (lvl2 → Premium Flower
Bread at Claw Game Cooker, not super_wheatmeal at Carousel Mill). All sell for coins.

**Crafting Table — implemented (Normal Mode only), full rewrite from old stale 3-item data to
21 items across 5 levels:**

| Item | Level | Ingredients | Workload | E-duration | Value | Module |
|---|---|---|---|---|---|---|
| Shell Ornament | 1 | 12 Shell | 18 | 18s | 177 coins | — |
| Wood Sculpture | 1 | 12 Willow Wood | 18 | 18s | 141 coins | — |
| Premium Wood Sculpture | 1 | 12 Willow Wood | 18 | 18s | 42 Bud Tickets | crafting_module:1 |
| Bamboo Ware | 2 | 4 Bamboo | 23 | 18s | 344 coins | — |
| Wind Chime | 2 | 1 Cotton Thread, 12 Shell | 45 | 36s | 585 coins | — |
| Pottery | 2 | 10 Clay, 12 Scales | 45 | 36s | 916 coins | — |
| Advanced Wind Chime | 2 | 1 Cotton Thread, 12 Shell | 45 | 36s | 176 Bud Tickets | crafting_module:2 (assumed — see below) |
| Pearl Necklace | 3 | 2 Pearl | 27 | 18s | 1197 coins | — |
| Bubble Bracelet | 3 | 2 Love Bubble | 27 | 18s | 1197 coins | — |
| Porcelain | 3 | 10 Clay, 4 Quartz | 54 | 36s | 1854 coins | — |
| Woven Toy | 3 | 1 Palm Rope, 4 Bamboo | 54 | 36s | 1633 coins | — |
| Bracelet | 3 | 1 Cotton Thread, 1 Palm Rope | 54 | 36s | 1622 coins | — |
| Dye | 3 | 5 Lavender, 6 Rose | 54 | 36s | 2523 coins | — |
| Premium Woven Toy | 3 | 1 Palm Rope, 4 Bamboo | 54 | 36s | 490 Bud Tickets | crafting_module:3 |
| Dream Catcher | 4 | 1 Cotton Thread, 1 Dried Flowers, 2 Natural Rubber | 108 | 1m12s | 6306 coins | — |
| Gemstone Dust | 4 | 2 Gem | 27 | 18s | 1938 coins | — |
| Advanced Dream Catcher | 4 | 1 Cotton Thread, 1 Dried Flowers, 2 Natural Rubber | 108 | ? | 1892 Bud Tickets | crafting_module:4 (assumed — see below) |
| Flowers in a Bottle | 5 | 1 Porcelain, 1 Dried Flowers, 1 Rose Freshener | 162 | 1m12s | 11834 coins | — |
| Bouquet | 5 | 1 Cotton Fabric, 1 Dried Flowers | 81 | 1m48s | 5721 coins | — |
| Starwish Lantern | 5 | 2 Star, 1 Gemstone Dust, 12 Scales | 81 | 54s | 5295 coins | — |
| Doll | 5 | 1 Wool Fabric, 1 Dye, 6 Cotton | 108 | 1m12s | 6758 coins | — |

**Open question — "Advanced" vs "Premium" naming:** section 10's Crafting Module table names
the lvl2/lvl4 unlocks "Premium Wind Chime" and "Premium Dream Catcher," but this item table
calls them "Advanced Wind Chime" and "Advanced Dream Catcher." Assumed these are the same items
(module gating applied: crafting_module:2 and crafting_module:4 respectively, matching table
position), but not explicitly confirmed — could be inconsistent naming from the user's source,
or could genuinely be two different item tiers. Worth double-checking later.

**New referenced-but-undefined items** (ingredients used here that don't resolve to any
ProductionItem yet — recipes needing them will simply be filtered out as unavailable by the
optimizer until defined, same graceful-degradation behavior as the Aniipod Maker rock/copper
gap): Cotton Thread, Palm Rope, Pearl (Tidewhisper Sandcastle), Love Bubble, Cotton Fabric,
Wool Fabric, Rose Freshener, Star, Scales (Grass Blossom Mat — defined in section 12 but not
yet wired into `FacilityCounts`/code, so still unresolvable for now), Dried Flowers (defined in
old stale `jukebox_dryer.csv`, not yet updated for new beta).

**Code changes this pass:**
- `ProcessingRowWithEnergy` and `ProcessingRowNoEnergy` (`models.rs`) now accept either
  `production_time` (old-style, still used by not-yet-updated Jukebox Dryer/Dance Pad
  Polisher/Aniipod Maker) or `workload` (new-beta style) — at least one required, loader derives
  time via `WORKLOAD_RATE_ESTIMATE` when workload is present.
- `ProcessingRowNoEnergy` gained an optional `sell_currency` column (defaults preserved
  per-facility: "coins" for Aniipod Maker, "coupons" for Dance Pad Polisher, matching prior
  hardcoded behavior, until those are updated with real new-beta data).
- Noted a **pre-existing bug** (not introduced by this work): `data.rs` (CLI) and `wasm.rs`
  (web) already disagreed on default sell currency for `load_processing_no_energy` before any
  of these changes — CLI hardcoded "coins" unconditionally, web differentiated by facility
  ("coupons" for Crafting Table/Dance Pad Polisher, "coins" for Aniipod Maker). Preserved as-is
  for now since it's out of scope, but worth fixing whenever those facilities get real data.
- `tests/data_tests.rs` currency assertion updated to allow `bud_tickets` alongside
  `coins`/`coupons`.

### 15. Claw Game Cooker — data received but BLOCKED on missing ingredients

User-provided table (Ingredients column was blank for every row — **cannot implement without
this**, would make every item look free to the optimizer):

| Item | Level | Workload | E-duration | Value | Energy |
|---|---|---|---|---|---|
| Malt Sugar | 1 | 23 | 18s | 150 | 12600 |
| Rock Candy | 1 | 23 | 18s | 210 | 12600 |
| Sugar-Roasted Chestnuts | 1 | 23 | 18s | 303 | N/A |
| Flower Bread | 1 | 45 | 36s | 540 | N/A |
| Advanced Flower Bread | 1 | 45 | 36s | 162 Bud Tickets | N/A |
| Strawberry Candy | 2 | 45 | 36s | 900 | 25920 |
| Tanghulu | 2 | 45 | 36s | 960 | 25920 |
| Maple Candy | 2 | 23 | 18s | 783 | 12960 |
| Maple Candy Star | 2 | 45 | 36s | 1719 | 25920 |
| Strawberry Cream Puff | 2 | 45 | 36s | 1650 | 25920 |
| Tofu Cake | 2 | 45 | 36s | 1155 | 25920 |
| Soy Sauce Tofu (typo'd "Tofy") | 2 | 45 | 36s | 1425 | 25920 |
| Soy Sauce Fried Rice | 2 | 45 | 36s | 1095 | 25920 |
| Creamy Potato Bisque | 2 | 45 | 36s | 735 | 25920 |
| Advanced Soy Sauce Tofu | 2 | 45 | 36s | 428 Bud Tickets | N/A |
| Coconut Cookie | 3 | 54 | 36s | 2100 | 30090 |
| Grape Candy | 3 | 54 | 36s | 2700 | 30090 |
| Jello | 3 | 81 | 54s | 4140 | 45130 |
| Pumpkin Rice Cake | 3 | 54 | 36s | 2775 | N/A |

**Confirms the "Advanced X = Premium X" naming pattern** from section 14 — Advanced Flower
Bread and Advanced Soy Sauce Tofu match the Kitchen Module's "Premium Flower Bread"/"Premium
Soy Sauce Tofu" unlocks (section 10), same inconsistent-labeling pattern as Crafting Table's
Advanced Wind Chime/Advanced Dream Catcher. Now treating this as confirmed, not just a guess.

**RESOLVED — ingredients received, fully implemented.** Full 19-item ingredient list:

| Item | Level | Ingredients |
|---|---|---|
| Malt Sugar | 1 | 75 Wheat |
| Rock Candy | 1 | 15 Sugarcane |
| Sugar-Roasted Chestnuts | 1 | 6 Chestnut |
| Flower Bread | 1 | 3 Wheatmeal, 6 Petals |
| Advanced Flower Bread | 1 | 3 Wheatmeal, 6 Petals (kitchen_module:2) |
| Strawberry Candy | 2 | 1 Malt Sugar, 5 Strawberry |
| Tanghulu | 2 | 1 Rock Candy, 5 Strawberry |
| Maple Candy | 2 | 6 Maple Syrup |
| Maple Candy Star | 2 | 1 Maple Syrup, 2 Star |
| Strawberry Cream Puff | 2 | 1 Coconut Oil, 1 Dried Strawberry |
| Tofu Cake | 2 | 1 Tofu, 1 Rock Candy |
| Soy Sauce Tofu | 2 | 1 Soy Sauce, 1 Tofu |
| Soy Sauce Fried Rice | 2 | 1 Soy Sauce, 1 Rice (product) |
| Creamy Potato Bisque | 2 | 1 Wheat Tea, 30 Potato |
| Advanced Soy Sauce Tofu | 2 | 1 Soy Sauce, 1 Tofu (kitchen_module:3) |
| Coconut Cookie | 3 | 3 Wheatmeal, 1 Shredded Coconut |
| Grape Candy | 3 | 1 Malt Sugar, 5 Grapes |
| Jello | 3 | 5 Grapes, 3 Wheatmeal, 1 Shredded Coconut |
| Pumpkin Rice Cake | 3 | 1 Pumpkin, 1 Rice (product) |

Confirms the Advanced/Premium naming pattern a third time (now treated as fully established):
Advanced Flower Bread = Premium Flower Bread (kitchen_module:2), Advanced Soy Sauce Tofu =
Premium Soy Sauce Tofu (kitchen_module:3), matching section 10's Kitchen Module table exactly.

Most ingredients resolve to items already in the data (Wheat, Sugarcane, Chestnut, Wheatmeal,
Petals, Strawberry, Maple Syrup, Coconut Oil, Tofu, Rice→`rice_processed`, Potato→`potatoes`,
Grapes→`grape`, Pumpkin, plus self-references to Malt Sugar/Rock Candy within this same table).

**Still-undefined ingredients** (recipes needing them are simply unavailable until these exist
somewhere): ~~Star~~, ~~Dried Strawberry~~, ~~Soy Sauce~~, ~~Wheat Tea~~, ~~Shredded Coconut~~ —
all now resolved: Dried Strawberry/Shredded Coconut via Jukebox Dryer (section 17), Soy
Sauce/Wheat Tea via Bouncy Brew Keg (section 18). Star remains undefined (needed by Crafting
Table's Starwish Lantern and Maple Candy Star here) — no known source facility yet.

**Code changes:** `ProcessingRowWithEnergy` (shared by Carousel Mill/Jukebox Dryer/Claw Game
Cooker) gained the same optional `sell_currency` column Crafting Table's row type got in
section 14, needed for the two Bud-Ticket items here. New `data/claw_game_cooker.csv`, plus one
loader call each in `data.rs` and `wasm.rs` (both just reuse the existing
`load_processing_with_energy`/inline-equivalent — no new parsing logic needed). Added to the
`FACILITIES` config in `web/app.js`.

Claw Game Cooker is now fully implemented (Normal Mode) — third proof point for the new
generic facility architecture (section 16), and the first case where adding a facility required
data changes only, no additional Rust plumbing beyond the shared `sell_currency` field.

### 16. Architecture refactor: generic facility system

With 11 new facilities discovered (section 11) and more data arriving per-facility, the old
approach (named struct fields for each of the original 9 facilities, touched in ~6 places per
facility: `models.rs` struct + match arms, `wasm.rs` JSON struct + 2 construction sites, CLI args
in `main.rs`, HTML markup, JS input handling) would not have scaled. Refactored to a generic,
name-keyed system instead:

- **`FacilityCounts`** (`models.rs`): now backed by a `HashMap<String, (u32, u32)>` instead of
  9 named fields. Public API (`get_count`/`get_level`/`can_produce`) unchanged, so
  **`optimizer.rs` needed zero changes** (it already only used these string-keyed methods, never
  direct field access). New constructors: `FacilityCounts::from_pairs(&[("Farmland", 4, 3), ...])`
  and `FacilityCounts::show_all_levels()` (replaces the old hardcoded "(1, 99) for every facility"
  fallback in `get_available_items`).
- **`JsOptimizeInput`** (`wasm.rs`): `facilities` field is now `HashMap<String, JsFacilityConfig>`
  instead of 9 named fields. The map key must exactly match the Rust `facility` string (e.g.
  "Farmland", "Mineral Pile") — no separate name-mapping layer.
- **Web UI** (`web/app.js`, `web/index.html`): facility cards are now generated dynamically from
  a `FACILITIES` config array in `app.js`, instead of hand-written HTML blocks per facility.
  Adding a new facility to the UI going forward is a single array entry, not edits across 3 files.
- **CLI** (`main.rs`): kept explicit named `--farmland`/`--farmland-level` style args (reasonable
  for a CLI), just updated construction to use `FacilityCounts::from_pairs`.
- All existing tests and doctests updated to the new construction API.
- Added a "Bud Tickets" option to the currency `<select>` in the web UI, since Crafting Table now
  has Bud Ticket items.

**Grass Blossom Mat implemented as the first new facility using this system**, proving it works
end-to-end: new `data/grass_blossom_mat.csv` (scales/quick_scales, sell value 28, workload 2250),
reuses the Mineral Pile loading logic (generalized `load_workload_raw_material` in `data.rs`,
parallel inline block in `wasm.rs`). **Facility level is an unconfirmed guess (assumed Lvl 1)**
since the user hasn't told us — flagged in code comments and the UI tooltip.

Claw Game Cooker remains blocked on missing ingredients (section 15) — not added to
`FACILITIES`/data files yet, but adding it once ingredients arrive is now a small, contained
change (one CSV file + one array entry + one `wasm.rs`/`data.rs` loader block), not another
multi-file refactor.

### 17. Jukebox Dryer (updated) and Phonolfactory Table (new) — implemented

**Jukebox Dryer** — full rewrite from stale pre-beta data:

| Item | Level | Ingredients | Workload | Value | Energy |
|---|---|---|---|---|---|
| Potato Chips | 1 | 30 Potatoes | 18 | 195 | 13710 |
| Dried Strawberries | 2 | 5 Strawberry | 23 | 480 | 12520 |
| Dried Lemon Slices | 2 | 5 Lemon | 23 | 345 | 12520 |
| Dried Bean Curd | 3 | 1 Tofu | 27 | 870 | 13800 |
| Dried Flowers | 3 | 5 Lavender, 6 Rose | 54 | 2523 | N/A |
| Shredded Coconut | 3 | 5 Coconut | 27 | 900 | 13800 |
| Nuts | 4 | 3 Walnut, 6 Chestnut | 54 | 2778 | 30090 |
| Herbs | 4 | 2 Ginseng | 27 | 2040 | 15040 |
| Dried Grapes | 4 | 5 Grape | 27 | 2040 | 15040 |
| Caramel Nut Chips | 4 | 1 Nuts, 6 Maple Syrup | 81 | 4896 | 45130 |

Old `premium_dried_strawberry`/`high_grade_herbs` (kitchen_module gated) dropped — not present
in new table, and Kitchen Module's 4 slots are already fully accounted for elsewhere (section
10: feeding, Claw Game Cooker ×2, Bouncy Brew Keg), so Jukebox Dryer no longer has any
Kitchen-Module-gated items. This update resolves several previously-undefined ingredients used
elsewhere: `dried_flowers`, `shredded_coconut`, `dried_strawberry` (used by Claw Game Cooker's
Strawberry Cream Puff and Crafting Table's Bouquet/Flowers in a Bottle).

**Phonolfactory Table** (brand new facility) — 14 items across 4 levels:

| Item | Level | Ingredients | Workload | Value |
|---|---|---|---|---|
| Lemon Incense | 1 | 5 Lemon | 23 | 345 coins |
| Lavender Incense | 2 | 5 Lavender | 27 | 1260 coins |
| Rose Incense | 2 | 6 Rose | 27 | 1263 coins |
| Deluxe Lavender Incense | 3 | 1 Lavender Incense, 4 Aromathyst | 54 | 3708 coins |
| Premium Rose Incense | 3 | 1 Rose Incense, 4 Aromathyst | 54 | 3711 coins |
| Premium Lemon Incense | 3 | 1 Lemon Incense, 4 Aromathyst | 54 | 2793 coins |
| Soap | 3 | 1 Coconut Oil, 1 Lemon Incense | 54 | 2145 coins |
| Rose Freshener | 3 | 4 Bamboo, 1 Rose Incense | 54 | 2507 coins |
| Cedarwood Incense | 3 | 1 Pine Tree Hardwood | 27 | 2040 coins |
| Sachet | 4 | 1 Soap, 1 Cotton Fabric | 81 | 5343 coins |
| Lotion | 4 | 1 Coconut Oil, 1 Soap | 81 | 5400 coins |
| Deluxe Cedarwood Incense | 4 | 1 Cedarwood Incense, 4 Aromathyst | 54 | 5118 coins |
| Mixed Perfume | 4 | 1 Cedarwood Incense, 1 Coconut Oil | 54 | 4470 coins |
| Coconutty Candle | 4 | 1 Mixed Perfume (qty assumed), 1 Pine Tree Hardwood | 81 | 8475 coins |

**Important distinction from other facilities' "Advanced/Premium" pattern:** these
"Deluxe X"/"Premium X" items sell for plain **coins**, not Bud Tickets — unlike every other
Advanced/Premium item seen so far (all of which paired with Bud Ticket pricing and a module
gate). So these are NOT module-gated; they're just alternate recipes that consume a rare
ingredient (Aromathyst). Left `module_requirement` blank for all of them — no evidence any of
the four known modules (Ecological/Kitchen/Mineral Detector/Crafting) gate these.

This resolves `rose_freshener` (previously undefined, needed by Crafting Table's "Flowers in a
Bottle"). New undefined ingredient introduced: **Aromathyst** — no known source facility yet.
`Cotton Fabric` (used by Sachet) remains undefined from before.

**Assumption:** Coconutty Candle's Mixed Perfume quantity wasn't given in the source table
(just "Mixed Perfume" with no leading number) — assumed 1, consistent with other single-unit
crafting inputs. Worth double-checking.

**Code changes:** none needed beyond loader wiring — both facilities reuse the
`ProcessingRowWithEnergy`/`ProcessingRowNoEnergy` structs already generalized in sections 14-15
(workload + optional sell_currency). New `data/phonolfactory_table.csv`, one loader call each in
`data.rs`/`wasm.rs`, one `FACILITIES` entry in `app.js`. Confirms the architecture refactor is
paying off as intended — this facility took data + 3 small wiring edits, no struct changes.

### 18. Bouncy Brew Keg — implemented, resolves soy_sauce and wheat_tea

Brand new facility, 10 items across 3 levels:

| Item | Level | Ingredients | Workload | Value |
|---|---|---|---|---|
| Soy Sauce | 1 | 8 Soybean | 23 | 480 coins |
| Sweet Rice Drink | 1 | 1 Rice (product) | 23 | 345 coins |
| Strawberry Jam | 1 | 5 Strawberry | 23 | 480 coins |
| Wheat Tea | 1 | 75 Wheat | 23 | 195 coins |
| Tequila Highball | 2 | 4 Agave | 27 | 1259 coins |
| Fermented Rice Drink | 2 | 1 Sweet Rice Drink | 27 | 660 coins |
| Potato Kvass | 2 | 1 Rock Candy, 30 Potato | 54 | 990 coins |
| Advanced Potato Kvass | 2 | 1 Rock Candy, 30 Potato | 54 | 297 Bud Tickets |
| Grape Juice | 3 | 5 Grape, 1 Wheat Tea | 54 | 3375 coins |
| Tequila Soda | 3 | 1 Grape Candy, 1 Tequila Highball | 81 | 6434 coins |

Advanced Potato Kvass confirmed `kitchen_module:4` — fourth confirmation of the Advanced/Premium
pattern, exactly matching section 10's Kitchen Module table (Lvl 4 → Premium Potato Kvass at
Bouncy Brew Keg).

**Resolves two long-standing undefined ingredients: `soy_sauce` and `wheat_tea`** — this
unblocks several previously-unavailable Claw Game Cooker recipes (Soy Sauce Tofu, Advanced Soy
Sauce Tofu, Soy Sauce Fried Rice, Creamy Potato Bisque).

**Cross-facility ingredient chains**: Potato Kvass/Advanced Potato Kvass need Rock Candy (Claw
Game Cooker), Tequila Soda needs Grape Candy (Claw Game Cooker) — confirms recipes can depend on
other new facilities, and the optimizer's existing multi-level chain support handles this
correctly without any code changes.

**Code changes:** none beyond the usual 3-point wiring (CSV + one loader call each in
`data.rs`/`wasm.rs` + one `FACILITIES` entry) — architecture refactor continuing to pay off.

### 19. Tidewhisper Sandcastle, Starfall Hammock, Dewy House — items identified, BLOCKED on sell values

User-provided tables (extrapolated from recipe references in other facilities' ingredient
lists):

| Facility | Item | Aniimo Family | Workload | Output | Environment |
|---|---|---|---|---|---|
| Tidewhisper Sandcastle | Pearl | Shelly | 2700 | 2 | Cool |
| Tidewhisper Sandcastle | Love Bubble | Susuta | 2700 | 2 | Freeze |
| Starfall Hammock | Star | Celestis | 2250 | 2 | Cool |
| Dewy House | Aromathyst | Dewy | 2700 | 4 | Warm |

**Resolves 4 previously-undefined ingredients**: Pearl (unblocks Crafting Table's Pearl
Necklace), Love Bubble (unblocks Bubble Bracelet), Star (unblocks Maple Candy Star, Starwish
Lantern), Aromathyst (unblocks Phonolfactory Table's Deluxe/Premium incense tier + Sachet/Lotion
chain).

**Significant environment finding:** unlike Grass Blossom Mat (mostly "N/A" environment, one
"Adequate"), *every* item here has a real, specific environment requirement (Cool/Freeze/Warm).
Combined with the Heat Furnace/Cooling Unit/Sunlamp Auxiliary Facilities (section 11), this
strongly suggests these three Materials facilities *require* a matching climate-control building
to function at all — not just a minor bonus like Quick Scales' "Adequate" might be. "Cool" vs
"Freeze" for the two Tidewhisper Sandcastle items could mean Cooling Unit has intensity tiers.
**Not yet modeled as a hard gate** — no data yet on how Heat Furnace/Cooling Unit actually work
(cost, unlock, whether count/level matters), so recording environment as informational only for
now, consistent with how "Adequate" was handled.

**RESOLVED — sell values received, all three facilities implemented:** Pearl 441, Love Bubble
441, Star 273, Aromathyst 357 (all coins). Facility level assumed 1 for all (unconfirmed, same
caveat as Grass Blossom Mat). Same pattern as Mineral Pile/Grass Blossom Mat (no cost, workload/
Aniimo-family driven) — `data/tidewhisper_sandcastle.csv`, `data/starfall_hammock.csv`,
`data/dewy_house.csv`, each with a thin `load_workload_raw_material` wrapper in `data.rs` and a
matching inline block in `wasm.rs`, plus `FACILITIES` entries in `app.js`.

This resolves the last of the previously-undefined ingredients from Crafting Table (Pearl
Necklace, Bubble Bracelet, Maple Candy Star, Starwish Lantern) and Phonolfactory Table's
Aromathyst-dependent tier (Deluxe/Premium incense, Sachet, Lotion, Deluxe Cedarwood Incense,
Mixed Perfume, Coconutty Candle) — those recipes should now resolve as producible in the
optimizer once rebuilt, assuming the environment requirement isn't actually a hard blocker
in-game (see caveat above — not yet confirmed either way).

### 20. Dance Pad Polisher and Aniipod Maker removed entirely

Per user instruction: these facilities don't produce coins/coupons/Bud Tickets in the new beta,
so they're out of scope for this optimizer. Fully removed rather than left stale:
- Deleted `data/dance_pad_polisher.csv`, `data/aniipod_maker.csv`.
- Removed their loader calls from `data.rs` (`load_all_data`) and inline blocks from `wasm.rs`
  (`get_embedded_items`).
- Removed their CLI args, `FacilityCounts` entries, and summary `println!`s from `main.rs`.
- Removed their entries from doctests (`lib.rs`, `optimizer.rs` ×2) and test fixtures
  (`models_tests.rs`, `optimizer_tests.rs` ×4 call sites).
- Removed their `FACILITIES` config entries from `web/app.js`.

Net effect: 14 facilities now implemented with real new-beta data, out of 14 total in scope
(20 minus the 6 Auxiliary Facilities, which are deferred — see section 11/16). Every
currency-producing facility we know about is now implemented.

### 21. Currency set finalized: Coins + Bud Tickets only (coupons removed entirely)

Resolves open question #4. Confirmed: coupons no longer exist as a currency at all — fully
replaced by Bud Tickets. Removed every remaining "coupons" reference across the codebase (doc
comments in `models.rs`/`optimizer.rs`/`lib.rs`/`main.rs`/`data.rs`/`wasm.rs`, the
`test_currency_types` assertion in `data_tests.rs`, the Coupons `<option>` in the web currency
dropdown and help text). Repurposed `test_calculate_efficiencies_coupons` into
`test_calculate_efficiencies_bud_tickets` (had to unlock Crafting Module in that test since all
Bud Ticket items are module-gated Premium/Advanced recipes — with default modules the result set
would have been empty and the test would have been vacuous).

No CSV data needed changing — nothing was ever actually written with `coupons` as a data value
(only doc comments and now-obsolete Dance Pad Polisher/Aniipod Maker assumptions, both already
gone).

### 22. Bug fix: ingredient auto-substitution was still looking for `high_speed_` items

Found while starting the multi-resource work: `optimizer.rs` has logic (4 call sites) that
automatically substitutes a recipe's raw material with its higher-yield module-gated variant
when available — but it was still constructing lookup names as `high_speed_{item}`, the OLD
naming convention. Since all new-beta upgraded items are named `quick_{item}` (section 4), this
substitution has been silently finding nothing and always falling back to the base item for the
whole session, even when the right module was unlocked. Fixed all 4 call sites to use `quick_`.
This affects every recipe chain that consumes wheat/lemon/coconut/rose/shell/quartz etc. as an
ingredient — those chains will now correctly prefer the Quick variant when the corresponding
module is unlocked, which changes (improves) profit/time numbers for anything using them as
inputs. Not yet rebuilt/retested at the time of this note — will be covered by the next full
rebuild.

### 23. New feature: Homeland Upgrade multi-resource optimizer

User's real need: RV/Homeland level-ups cost coins + Wood Blocks + Mineral Sand simultaneously
(example given: level 6 needs 6900 Wood Blocks, 3500 Mineral Sand, 120000 coins), and the
existing single-currency optimizer can't answer "what should I produce to hit all three
fastest, given my current balances." User confirmed the "fully integrated" approach: treat the
Resource Exchange (section 7) as a genuine lever inside the same solver, not a separate/deferred
step.

**Algorithm** (`find_homeland_upgrade_path` in `optimizer.rs`): binary search on total time `T`.
- Whichever item has the best Wood-Blocks/sec rate gets its facility type fully dedicated to it
  for the whole duration `T` (coin side-income counts toward the coin target for free).
- Same for whichever item has the best Mineral-Sand/sec rate. Byproduct-item selection is
  generic — it filters by `item.byproduct`'s resource name, so it automatically picks up new
  facilities later if they ever get confirmed byproduct data, no code changes needed.
- Any remaining Wood Block / Mineral Sand shortfall at time `T` can be bought via the Resource
  Exchange, capped at `EXCHANGE_DAILY_CAP_UNITS * (T / SECONDS_PER_DAY)` per resource (prorated
  daily cap, not exact day-boundary accounting).
- Every other facility (not already dedicated above) produces coins directly via the single best
  item among them (reuses `calculate_efficiencies` filtered to exclude the dedicated facilities).
- All quantities are monotonic in `T`, so binary search finds the minimum feasible `T`.

**Known simplifications** (documented in the function's doc comment too):
- Single-best-item-per-resource, not a full LP blend across multiple facilities/items — matches
  the rest of the codebase's greedy design philosophy (`find_best_production_path` already works
  the same way for a single currency target).
- Exchange sell side (5 coins/unit) isn't modeled — this mode assumes you want to keep the
  resources you produce, not sell them back.
- Hardcoded to coins specifically (not parameterized by currency) since Homeland upgrades and
  the Resource Exchange both deal in coins, not Bud Tickets.
- Exchange daily cap is prorated continuously (`cap × T/86400`) rather than modeling exact
  day-by-day purchase timing — a reasonable approximation for a "how long will this take"
  estimate, but won't exactly match in-game day-boundary mechanics.

**New surface area:**
- `models.rs`: `HomelandUpgradePath`, `HomelandUpgradeStep` structs; `EXCHANGE_BUY_RATE_COINS_PER_UNIT`
  (20.0), `EXCHANGE_DAILY_CAP_UNITS` (2000.0), `SECONDS_PER_DAY` constants.
- `optimizer.rs`: `find_best_byproduct_item` (helper) + `find_homeland_upgrade_path` (public).
- `wasm.rs`: `JsHomelandUpgradeInput`/`JsHomelandUpgradeStep`/`JsHomelandUpgradeResult` +
  `optimize_homeland_upgrade` WASM export.
- `web/index.html` + `web/app.js`: new "Homeland Upgrade Calculator" section (target + current
  balance inputs for coins/Wood Blocks/Mineral Sand, its own Calculate button and results
  display), reusing the same Facilities/Modules configuration as the main calculator. Refactored
  `getInputValues()` to share a `getFacilitiesAndModules()` helper with the new
  `getHomelandUpgradeInputValues()`.

**Follow-up: unified into a single calculator.** User feedback after the first version: don't
keep two separate calculators (the old single-currency one + the new multi-resource one) — there
should be one calculator where any of the three resources (coins, Wood Blocks, Mineral Sand) can
be targeted, leaving unneeded ones at 0.

- Fixed a real bug this surfaced: the solver was **always** dedicating Woodland/Mineral Pile to
  byproducts even when their target was 0 (e.g. a coins-only query), forcing potentially
  worse-for-coins items instead of letting those facilities compete normally. Now only dedicates
  a facility type to a byproduct when `delta > 0` for that resource — a coins-only query now
  behaves exactly like a plain single-currency optimization, using every facility.
- Retired the old single-currency web UI entirely (`optimize()`/`JsOptimizeInput` and friends in
  `wasm.rs` are still there and still used by the CLI's `main.rs` — not deleted — just no longer
  called from `app.js`). Removed Target Amount/Currency dropdown, Energy Cost, Energy
  Self-Sufficient, and Cross-Facility Parallel Production from the web UI (they didn't map
  cleanly onto the multi-resource model). Kept Exclude Wheat, threaded through to
  `optimize_homeland_upgrade` via a new `exclude_wheat` field on `JsHomelandUpgradeInput`.
- **Bud Tickets deliberately excluded** from the unified calculator (not just deferred) — Crafting
  Table and Claw Game Cooker can produce either coin or Bud-Ticket items from the *same*
  facility, and modeling that shared-facility tradeoff alongside coins/Wood Blocks/Mineral Sand
  is a bigger problem than what's needed right now. Noted in the UI so it's not a silent gap.
- One Configuration section, one Results section: Target/Current Coins, Wood Blocks, Mineral
  Sand pairs replace the old Target Amount/Currency fields; results always show Total Time +
  all three produced amounts + a unified "What To Produce" step list (dedicated items +
  Exchange purchases). Verified live: a coins-only query (Wood Blocks/Mineral Sand left at 0)
  no longer force-dedicates Woodland/Mineral Pile.

**Rebuilt and verified live** — 28/28 tests pass, WASM builds clean. Two scenarios confirmed
through the unified calculator:
- Coins-only (Wood Blocks/Mineral Sand left at 0): Woodland/Mineral Pile correctly stay in the
  general coin-producing pool instead of being forced onto byproduct items — confirms the
  dedication bugfix works as intended.
- Full 3-resource query (the level-6 example: 120,000 coins / 6,900 Wood Blocks / 3,500 Mineral
  Sand): identical result to the pre-merge version (14h4m25s, Woodland→Pine, Mineral Pile→Clay,
  Bouncy Brew Keg→Strawberry Jam, ~1,173 Mineral Sand bought via Exchange for 23,456 coins) —
  confirms the merge didn't change the underlying algorithm's behavior, only the UI wiring.

### 24. Security fix: removed polyfill.io script tag

User reported a browser HTTP Basic Auth ("Sign in") popup for `https://polyfill.io` on first
load. Traced to a leftover `<script src="https://polyfill.io/v3/polyfill.min.js?features=es6">`
tag in `web/index.html` (original project boilerplate). polyfill.io was acquired in 2024 by an
operator that injected malicious code into the scripts it served, and the domain has been
blocked/flagged by browsers and CDNs since — an unexpected login prompt from it now is not
something to authenticate against. **User was told to click Cancel, not enter anything.**
Removed the script tag entirely: unnecessary anyway, since the app already requires WebAssembly
+ ES modules, so any browser able to run it doesn't need an ES6 polyfill. Verified the app still
loads and works correctly (MathJax's CDN script, a legitimate/unrelated source, left in place).

### 25. Feature: auto-save inputs via localStorage

User asked whether values could be saved once this is hosted on GitHub Pages (prompted by
confusion over an unrelated browser login popup — see section 24). Answer: yes, `localStorage`
is scoped to the page's origin and works identically on `localhost` and GitHub Pages, no backend
or account needed (caveat: it's per-browser/per-device, not synced across devices).

Implemented in `web/app.js`: all Configuration/Facilities/Modules inputs auto-save to
`localStorage` (key `aniimax-config-v1`) on every change, and are restored on page load (after
`renderFacilityCards()` so the dynamically-generated facility inputs exist first). Added a
"Clear saved values" button (`web/index.html`) that wipes storage and reloads back to defaults.
Verified live: set values → survived a full page reload → Clear button correctly wiped storage
and reset to defaults. Pure JS/HTML change, no Rust/WASM rebuild needed.

### 26. Feature + bugfix: user-specified Resource Exchange availability

User wanted an input for currently-available currency conversions, "so we don't have to guess
whether the user has hit the daily [cap]." While implementing, found and fixed a real accuracy
bug: `EXCHANGE_DAILY_CAP_UNITS` was 2000 (derived from the **sell**-direction line, 200
units/exchange × 10), but the Homeland Upgrade optimizer uses the **buy** direction
("2000 Home Coin → 100 Wood Block"), which is only 100 units/exchange — each of the shop's 6
lines has its own independent 10/day cap, so the correct buy-side daily cap is **1000
units/day**, not 2000. Fixed in `models.rs` (`EXCHANGE_DAILY_CAP_UNITS`, new
`EXCHANGE_UNITS_PER_TRANSACTION` constant).

**New inputs**: "Wood Blocks Exchanges Left Today" / "Mineral Sand Exchanges Left Today" (0-10
each, default 10). `find_homeland_upgrade_path` now takes
`wood_blocks_exchanges_remaining_today`/`mineral_sand_exchanges_remaining_today` and models
Exchange availability as a **step function**, not continuous prorating: `start_units +
floor(T / 86400) × 1000` — today's actual remaining allowance is exact, every subsequent
24-hour period (from now, not from the real in-game reset time) adds another full day's cap.
Documented as an approximation in the function's doc comment (we don't know the actual reset
clock time, only "how much is left right now").

**Deliberately NOT auto-saved** via the localStorage persistence feature (section 25) — unlike
every other input, these two reset daily, so persisting them would risk silently restoring a
stale count from a previous day, reintroducing exactly the guessing problem this feature was
meant to solve. Always defaults to 10 on page load; user re-enters the real count each session.

**Verified live**: setting Mineral Sand Exchanges Left Today to 0 on the level-6 example
correctly dropped the Exchange purchase step entirely (falls back to pure production, since
direct output reaches the target in ~21h — faster than waiting a full day for the reset), vs.
the same query with a full 10 remaining still buying ~1,173 Mineral Sand via Exchange in ~14h.
28/28 tests pass, clean WASM build.

### 27. Bugfix: coin production now runs on every owned facility in parallel

User noticed the result only ever named a single coin-producing item/facility and suspected the
solver wasn't accounting for running everything simultaneously — correct. The old `other_rate`
was the single best `effective_profit_per_second` across ALL non-byproduct-dedicated items,
which silently assumed only one facility type would ever be producing coins, leaving every other
owned facility modeled as idle even though they could run in parallel for free.

**Fix**: `other_rate` is now the **sum** of the best coin-producing item's rate at each distinct
owned facility (not dedicated to Wood Blocks/Mineral Sand) — a Farmland, a Carousel Mill, and a
Claw Game Cooker all count simultaneously, not just whichever one is individually fastest. This
generally makes plans complete meaningfully faster than before.

**Result now reports every owned facility**, not just the top pick: `HomelandUpgradePath.coin_item`
(`Option`) became `coin_items` (`Vec`), one entry per owned facility not dedicated to a
byproduct. Facilities with nothing currently profitable to produce show up with `item_name: None`
("nothing to do") instead of being silently omitted — verified live this correctly surfaces real
constraints, e.g. Crafting Table/Jukebox Dryer/Phonolfactory Table going idle when their raw
materials (Shell, Lemon) aren't being produced because Mineral Pile/Woodland are fully dedicated
to Mineral Sand/Wood Blocks instead.

28/28 tests pass, clean WASM build, verified live with a 9-facility scenario — all appeared in
the result, three correctly flagged idle due to genuine raw-material conflicts.

### 28. UX: table output + fixed coin-production double-counting

Two pieces of feedback on the same screenshot: (1) the 12-item numbered step list was unwieldy —
wanted a table, one row per facility; (2) "Coins Produced: 97,502" looked wrong against a
~37,421 real deficit (120,000 target − 82,579 current) — "it's like it's not considering what I
currently have."

**Investigated (2) first, since it looked like a real bug — and it partly was.** `delta_coins`
itself was already computed correctly. The actual bug: `coins_produced` was crediting
`byproduct_coin_income` (Woodland/Mineral Pile's coin side-income over the *entire* Wood-Blocks-
driven duration) **plus** `remaining_coins_needed`, but the `coin_items` list separately credited
*every* owned facility as if all of them were actively producing for that same full duration —
double-counting: the reported total implied both "byproduct income already covers it" and "every
other facility is also grinding coins," when in this case only the first was true.

**Fixed with a proper minimal-selection pass**: after computing `remaining_coins_needed`,
greedily select the fewest highest-rate facilities whose combined output actually covers it
(sorted by rate descending) — if byproduct income alone already covers the deficit, **zero**
other facilities are selected. `coins_produced` now only credits byproduct income plus the
selected subset's output, not every owned facility unconditionally.

**Verified live** with the user's exact numbers: `coins_produced` stayed at 97,502.59 (same as
before the fix) — which turned out to be *correct*, not a leftover bug. In this specific setup,
Woodland (chestnut) + Mineral Pile (clay) running the full 16h42m needed for the Wood Blocks
target alone generate more coin side-income (97,502) than the 37,421 + 20,000 Exchange cost
(57,421) actually required — so **zero** other facilities are needed, and the table now says so
explicitly on every row ("Coin target is already covered without this facility") instead of
implying 6+ unrelated facilities all needed to be running. The fix's real effect here wasn't the
number — it was correctly reporting that most of the 12 old "steps" were never actually
necessary in the first place.

**New `HomelandUpgradeStepStatus` enum** (`Producing` / `NothingAvailable` / `NotNeeded`) added
to `HomelandUpgradeStep` along with a `reason: String`, so every row can explain itself.

**UI**: replaced the numbered step list with a table (`Facility | Count | Producing | Why`) in
`web/index.html`/`app.js`, with muted styling for `NotNeeded`/`NothingAvailable` rows
(`web/style.css`). Every owned facility gets exactly one row now, always.

28/28 tests pass, clean WASM build, verified live.

### 29. Feature: progress-over-time timeline + "long pole" callout, plus a real double-booking bugfix

Two asks: (1) "have a graph or timeline that shows when each value hits its goal, and be clear
about which resource is the long pole"; (2) a question — "it's telling us to just produce
chestnuts and not roast them at the jukebox, that seems odd, surely nuts is more profitable?"

**Timeline feature.** Added a client-side-only progress chart (`buildTimeline`/`renderTimeline`
in `web/app.js`) rendered as hand-built inline SVG, consistent with the rest of the app's
no-dependency approach. It reconstructs each targeted resource's balance-over-time curve from the
same totals the WASM solver already returns (`*_produced`, `*_bought`, `total_time_seconds`) —
production is a constant-rate ramp, Exchange purchases are modeled as front-loaded (bought as
early as the daily cap allows, since the solver only tracks final totals, not purchase timing).
Y-axis is % of each resource's own target so Coins/Wood Blocks/Mineral Sand — which have wildly
different absolute scales — can share one chart. Each line gets a crossing marker where it hits
100%, and whichever resource crosses last is called out by name as the "long pole" ("the other
target(s) finish earlier and the plan waits on this one"). Only resources with a nonzero target
are plotted. New CSS vars `--coins-color`/`--wood-color`/`--mineral-color` (dark+light) in
`web/style.css`. Verified live: for the user's saved scenario (120k coins / 6900 Wood Blocks /
3500 Mineral Sand), the chart correctly shows Mineral Sand done at 4h43m, Coins at 9h50m, and
**Wood Blocks as the long pole** at the full 16h42m14s — matching `total_time` exactly, as
expected since Wood Blocks is what Woodland is dedicated to for the whole run.

**The Jukebox question — investigated by testing the live scenario, not just reading code.**
Three separate prerequisites gate "nuts" (walnut + chestnut → 2778 coins, Jukebox Dryer lvl 4),
and the user's saved facility config was missing all three:
1. **Jukebox Dryer level.** Saved at level 1; "nuts" requires level 4. Below level 4 the
   optimizer literally cannot see the recipe — confirmed by reading the saved DOM state directly
   (`facility-jukebox-dryer-level` = "1").
2. **Woodland level, for walnut specifically.** Even after bumping Jukebox Dryer to level 4,
   "nuts" still didn't appear — walnut (an ingredient) also requires Woodland level 4, and the
   saved Woodland level was 2 (sufficient for chestnut, not walnut).
3. **Fertilizer.** Woodland level 3+ items (`requires_fertilizer: row.facility_level >= 3` in
   `src/data.rs`) — including both walnut and pine — need a Nimbus Bed to supply fertilizer.
   Saved Nimbus Bed count was 0, so even with both levels fixed, walnut (and pine) stayed
   filtered out (`calculate_efficiencies` skips any `requires_fertilizer` item when
   `nimbus_bed_count == 0`). Live-tested: with Jukebox Dryer lvl 4 + Woodland lvl 4 + Nimbus Bed
   count 2, the optimizer correctly switched Woodland to **rubber** (0.634 coins/sec/facility,
   even better than pine) and Jukebox Dryer to **caramel_nut_chips** (nuts + maple_syrup → 4896
   coins) — i.e. once the prerequisites are genuinely met, the solver does find and prefer the
   higher-tier recipes, including ones better than plain "nuts". So the original behavior was
   correct given the user's actual saved facility levels, not a bug — but see below.

**Real bug found and fixed along the way.** While tracing this, found that `other_items`/
`other_effs` (the "every facility not dedicated to Wood Blocks/Mineral Sand" pool in
`find_homeland_upgrade_path`, `src/optimizer.rs`) only excluded items whose own *processing*
facility was Woodland/Mineral Pile — not items processed elsewhere (e.g. Jukebox Dryer) that
*consume* Woodland/Mineral Pile output as ingredients. `calculate_efficiencies` computes a
processed item's ingredient-gathering rate using the *full* owned count of the raw material's
facility, with no awareness that Woodland might already be 100% committed to the Wood Blocks
target. So once a user's levels/fertilizer are sufficient to unlock "nuts", it would have been
double-booking Woodland — crediting its full chestnut+walnut output to "nuts" *and* separately
crediting Woodland's own direct-sale item in `best_per_facility`, overstating achievable coin
rate. Fixed by skipping any candidate item whose `all_facilities` set (already tracked per
`ProductionEfficiency` for exactly this kind of cross-facility bookkeeping) intersects
`excluded_facilities`. Confirmed this doesn't regress the "nuts found" scenario above, since in
that test Wood Blocks/Mineral Sand targets were 0 (nothing excluded).

**Known remaining limitation** (documented, not fixed): the same kind of contention can still
happen *between two "other" facilities* — e.g. a Jukebox item that consumes Farmland's potatoes
while Farmland's own best direct-sale item is also live in `best_per_facility`; both get credited
in parallel even though they compete for the same Farmland output. Solving this in general is a
much bigger allocation problem (splitting a raw facility's capacity between direct sale and one
or more downstream recipes) and is out of scope for now — flagging here so it isn't rediscovered
as a surprise later.

Also fixed a stale doc string in `web/index.html`'s help modal ("capped at 2000 units/day per
resource" — a leftover from the exchange-cap bug fixed in an earlier section; corrected to 1000).

28/28 tests pass, clean WASM build, verified live (including restoring the user's original saved
scenario afterward — testing this touched several saved localStorage fields).

### 30. Removed Wood Blocks/Mineral Sand entirely — back to a coins-only calculator

User realization mid-session: expanding plots in-game instantly grants a large amount of Wood
Blocks and Mineral Sand, making them not worth optimizing production for. Decision: strip the
multi-resource "Homeland Upgrade" solver back down to coins-only, removing the Resource Exchange
integration, byproduct dedication logic, and the timeline chart added in section 29 (built
specifically for the multi-resource "long pole" concept, which no longer applies with a single
target).

**Renamed** `HomelandUpgradePath`/`HomelandUpgradeStep`/`HomelandUpgradeStepStatus` →
`CoinPlan`/`CoinPlanStep`/`CoinPlanStepStatus` (`src/models.rs`) and
`find_homeland_upgrade_path` → `find_coin_plan` (`src/optimizer.rs`), since "Homeland Upgrade"
(an RV/Homeland level-up costing all three resources) no longer describes what the function does.
Removed `EXCHANGE_BUY_RATE_COINS_PER_UNIT`/`EXCHANGE_UNITS_PER_TRANSACTION`/
`EXCHANGE_DAILY_CAP_UNITS`/`SECONDS_PER_DAY` (models.rs) and `find_best_byproduct_item`
(optimizer.rs) — both now fully unused.

**The coins-only math is much simpler than what it replaced.** Without a Wood-Blocks-driven
duration forcing Woodland/Mineral Pile into a long dedicated run (which is what made some other
facilities' coin output "extra" and `NotNeeded` in section 28's fix), the fastest way to reach a
coin target is simply: sum the best coin rate across every owned facility (`total_rate`), then
`total_time = delta_coins / total_rate`. Since using more facilities in parallel only ever
shortens that time, the minimal plan always uses every owned, currently-profitable facility for
the full duration — so `CoinPlanStepStatus::NotNeeded` is no longer produced by this function
(kept in the enum for API stability, just currently unreachable). Deleted the greedy
minimal-facility-selection logic from section 28 entirely, since it existed only to handle the
"some facilities are extra" case that no longer arises.

**Not touched** (deliberately, to keep the change scoped and avoid unrelated churn): the
`byproduct: Option<(String, u32)>` field on `ProductionItem` and the `byproduct_yield` CSV column
for Woodland/Mineral Pile — these are now fully inert (nothing reads `.byproduct` anymore after
`find_best_byproduct_item` was deleted), but removing them would require touching every
`data.rs`/`wasm.rs` loader and CSV row struct for zero behavior change. Left as harmless dead
metadata; flagging here in case a future session wonders why it's unused.

**Web UI** (`web/index.html`, `web/app.js`, `web/style.css`): removed the Target/Current Wood
Blocks and Mineral Sand inputs, the Exchanges-Left-Today inputs and their hint text, the timeline
chart section and its CSS (`.timeline-*` classes, `--coins-color`/`--wood-color`/
`--mineral-color` vars), the Wood Blocks/Mineral Sand summary tiles, and the Resource Exchange
result row. Rewrote the Configuration intro copy, the Bud Tickets disclaimer, and the Help modal
to describe the coins-only behavior. `optimize_homeland_upgrade` (wasm.rs export) renamed to
`optimize_coin_plan`; `JsHomelandUpgradeInput`/`Step`/`Result` renamed to `JsCoinPlan*` to match.

**Not committed to git** — this whole session (the entire beta rework, not just this change) is
uncommitted working-tree changes on top of `fcf00f8` (last real commit, pre-beta). No git
checkpoint exists between "beta rework" and "beta rework minus Wood Blocks/Mineral Sand", so this
was done as a direct code edit rather than a revert.

28/28 tests pass, clean WASM build. Verified live: coin-only scenario runs correctly (all owned
facilities show `Producing`, no more `NotNeeded` rows), and the "already have enough" edge case
correctly shows "Nothing needed" with `total_time: 0`.

### 31. Bugfix: ingredient-dependency double-booking, plus simplified "why" reasons

User caught it live: the table recommended Farmland run strawberries (its own best standalone
item) while *also* recommending Bouncy Brew Keg run a rice drink — but rice drinks need rice,
which only Farmland produces, and Farmland was already fully committed to strawberries. Two
different, incompatible jobs assigned to the same facility at once.

**Root cause**: `find_coin_plan`'s `best_per_facility` picked the single best item independently
*per facility*, with no awareness that a processed item's rate (from `calculate_efficiencies`)
already assumes the *entire* owned count of its raw-material facility is dedicated to gathering
that ingredient. Bouncy Brew Keg's rice_drink rate was computed as if all 16 Farmlands grew rice
for it — while, completely independently, Farmland's own slot in `best_per_facility` was filled
with strawberries. Both got reported as `Producing` simultaneously, which isn't physically
possible with one Farmland. This is exactly the "known remaining limitation" flagged (but left
unfixed) at the end of section 29.

**Fixed** by replacing the per-facility-independent selection with a greedy conflict-free
set-packing pass — the same pattern already used elsewhere in this codebase by
`find_parallel_production_path` (and documented in the "How It Works" math modal: "Chains are
selected greedily by efficiency, skipping any that conflict with already-selected facilities").
Sort every candidate item across *all* facilities by `effective_profit_per_second` descending,
then walk the list: an item is selected only if every facility in `eff.all_facilities` (its own
facility, plus any raw-material/intermediate facilities its supply chain touches — already
tracked per `ProductionEfficiency`) is still unclaimed; selecting it claims all of them. Once
Claw Game Cooker's rock_candy (higher rate) claims both Claw Game Cooker *and* Farmland,
Farmland's own standalone candidates are skipped since Farmland is no longer free.

Verified live with the user's saved scenario: total time went from 1h3m48s to 2h49m15s (the old
number was inflated — it double-counted Farmland's output between its own item and whatever fed
off it). New plan: Farmland → sugarcane (feeds Claw Game Cooker's rock_candy), Woodland → lemon
(feeds Phonolfactory Table's lemon_incense), Mineral Pile/Grass Blossom Mat/Phonolfactory
Table/Claw Game Cooker sell directly, and Bouncy Brew Keg/Carousel Mill/Crafting Table/Jukebox
Dryer correctly show "No profitable item currently available" — their only worthwhile items
needed Farmland/Woodland, which higher-rate items already claimed.

**Also simplified the "why" text**, per the user's ask ("just say like 'sell directly' or
'funnels into jukebox dryer'"):
- A facility that sells its own item: `"Sells directly"`.
- A facility that's fully dedicated to supplying another facility's recipe: `"Funnels into
  {facility} ({item})"`, e.g. `"Funnels into Claw Game Cooker (rock_candy)"`. The "Producing"
  column shows the raw material name(s) (`eff.requires_raw`) rather than the downstream item, so
  the row reads as "what this facility is actually growing."
- A facility with nothing profitable available: `"No profitable item currently available"`
  (dropped the old parenthetical explanation — same simplification request).

**Known simplification carried over**: for a downstream item that pulls raw materials from
*multiple different* facilities, every one of those facilities' "Producing" column shows the same
joined `requires_raw` string (all ingredient names, not just the one this specific facility
grows), since that's the only name available without deeper per-facility disaggregation. Rare in
practice (most recipes pull from one facility) and not worth the added complexity right now.

**Update (still section 31, same session)**: this "simplification" turned out to be a visible bug,
not a rare edge case — user hit it immediately with pottery (Crafting Table: `clay;scales` from
Mineral Pile + Grass Blossom Mat). Both Mineral Pile's and Grass Blossom Mat's rows showed the
full joined string `"clay+quick_scales"`, making it look like each facility was growing *both*
materials. Fixed properly: built `item_facility: HashMap<&str, &str>` (item name → its facility,
from the full `items` list) and, for each ingredient-supplier row, filtered
`eff.requires_raw`'s `+`-separated names down to just the ones whose facility matches *this* row
before joining. Verified live by forcing pottery to actually win the greedy selection (temporarily
set Mineral Pile count to 1 so its solo clay rate no longer beat pottery's combined rate): Mineral
Pile's row now correctly shows `clay` / "Funnels into Crafting Table (pottery)", Grass Blossom
Mat's shows `scales` / "Funnels into Crafting Table (pottery)", each showing only its own
material.

28/28 tests pass, clean WASM build, verified live against the user's exact saved scenario.

### 32. Bugfix: coin plan now accounts for ingredient lead time, not pure simultaneity

User feedback: "the rice will have some lead up time to grow as a crop, then it will need to be
processed... we should consider not a pure simultaneous operation but some kind of hybrid."
`find_coin_plan` was computing `total_time = delta_coins / total_rate` — a closed-form calculation
that implicitly assumes every selected item is already at steady-state output from t=0, as if the
first rice_drink sells the instant the plan starts, with no time spent actually growing the rice
first.

**Fixed** with a proper hybrid model: every facility still starts working in parallel at t=0 (the
"simultaneous" part is real and stays), but each item's coin income doesn't begin until its own
first-batch lead time has passed (the "not pure simultaneous" part) — `rate * max(0, t -
lead_time)` per selected item, summed across all of them. This function is monotonically
non-decreasing and piecewise-linear in `t`, so the minimal `t` reaching `delta_coins` is found by
binary search (same doubling-then-bisecting pattern as the removed multi-resource version from
section 23/29).

**New `item_lead_time` helper** (`src/optimizer.rs`, right above `find_coin_plan`): recursively
computes true first-batch-ready time — `production_time` for a raw material (NOT divided by
facility count or yield; every owned copy finishes its first batch together, more facilities just
means more batches per completion, not a faster first one), or the slowest ingredient's own lead
time plus this item's own `production_time` for a processed item (recurses for multi-level chains
like nuts → caramel_nut_chips).

**Deliberately does NOT reuse `ProductionEfficiency::startup_time`** (which already exists and
looks like it should do this job) — traced through its definition and found it divides processing
time by facility count, which is correct for *steady-state throughput* (more processing facilities
= more batches per second) but wrong for *first-batch lead time* (more processing facilities
doesn't make the first batch finish faster). Reusing it would have understated lead times for
anyone owning 2+ of a processing facility. Also skips the small fertilizer add-on time for
simplicity (typically ~60s against multi-thousand-second production chains — noted as a known
minor simplification, not modeled).

Verified live against the user's saved scenario: total time increased from 2h49m15s to 3h13m54s
(~25 minutes) — the facility assignments and claims are unchanged, only the timing math is more
honest about the lead time before Farmland's sugarcane/Woodland's lemon are actually ready to feed
their downstream recipes. The "already have enough coins" edge case still correctly returns
`total_time: 0`.

28/28 tests pass, clean WASM build.

### 33. Feature: coin-income-over-time graph

User ask: "I want our result to include a graph where X is time and Y is profit. That way we can
visualize the influx of coins as batches are processed." Direct visual complement to section 32's
lead-time fix — the whole point of that fix was that coin income isn't a flat rate from t=0, so
it's worth actually showing the curve.

**Computed in Rust, not reconstructed in JS** (unlike the timeline chart built and later removed
in sections 29/30) — `find_coin_plan` already computes `coins_at(t)` internally for the binary
search, so `CoinPlan` gained a `timeline: Vec<(f64, f64)>` field: checkpoint `(time, cumulative
coins)` pairs at `0`, every distinct item lead time strictly within `(0, total_time)`, and
`total_time` itself. Since `coins_at` is piecewise-linear between kinks (each kink is where
another selected item's income turns on), these checkpoints reconstruct the curve *exactly* — no
dense sampling, and no risk of the chart drifting from the actual numbers the way a client-side
reconstruction could. Exposed through `wasm.rs`'s `JsCoinPlanResult.timeline`, which serializes
as `[[t, c], [t, c], ...]` (serde's default tuple-as-array encoding).

**`web/app.js`**: `renderTimeline(result)` draws a single-line SVG chart (time on X, cumulative
coins on Y) straight from `result.timeline` — literally just draws straight lines between the
checkpoints and dots at each one, no computation of its own. Hides itself if there are fewer than
2 points or nothing was actually produced (mirrors the "nothing needed" edge case). Reused the
`.timeline-*` CSS class names from the removed section-29 chart (simplified — no per-resource
colors or long-pole callout needed for a single line) since nothing else uses those names anymore.

**Caught and fixed a browser module-caching gotcha while verifying**: `<script type="module"
src="app.js">` (no cache-busting query) kept serving a stale cached copy after edits, even after
reloading the page and even after confirming via `fetch()` that the server was returning updated
content — reloading with a `?v=N` query on the *page* URL was what actually forced the module to
re-fetch. Not a code bug, just a reminder for testing during this kind of live-edit session.

Verified live: chart renders with the correct 5 checkpoints for the user's saved scenario (a flat
$0 segment during the initial lead-in, then rising kinks as sugarcane→rock_candy and lemon→
lemon_incense income come online), axes labeled with `formatNumber`/`formatTime`, and correctly
hidden for the "already have enough" edge case.

28/28 tests pass, clean WASM build.

### 34. Investigation + bugfix: multi-level chains ignored their own upstream bottleneck

User suspected the timing/workload model was fundamentally off and asked to clone the original
upstream repo (github.com/ae-bii/aniimax) as a reference, to check whether we'd strayed from a
previously-working approach. Cloned it to `../aniimax-reference` (sibling directory, read-only,
not part of this project). Findings:

- The reference repo's `calculate_efficiencies` — the steady-state `batches/sec = min(gathering
  rate, processing rate)` model — is **essentially identical** to ours. We didn't stray from the
  original math; we inherited it faithfully. The core "profit per second via steady-state
  bottleneck" approach has been unchanged since before this session's beta rework.
- "Workload" and "Efficiency Mode" don't appear anywhere in the reference repo — grepped the
  whole thing, zero hits. It predates those game mechanics entirely (its last commit, `fcf00f8`,
  is literally the same commit our working tree branched from). So it can't show us "the right
  way" to model workload — that mechanic simply didn't exist yet when it was written, and modeling
  it (section 15's `WORKLOAD_RATE_ESTIMATE`) was necessarily new territory this session, not a
  regression from something the original had already solved.
- **But a real, pre-existing bug turned up** while tracing through the user's specific example
  (rice at Farmland → rice_processed at Carousel Mill → sweet_rice_drink at Bouncy Brew Keg): the
  gathering-rate loop in `calculate_efficiencies` computes how fast an ingredient can be supplied
  using *only that ingredient's own* `yield_amount / production_time` — correct for a raw
  material, but wrong when the ingredient is itself a processed item. For rice_processed feeding
  sweet_rice_drink, the old code treated Carousel Mill's ~16s processing time as the ingredient
  supply rate, completely ignoring that rice_processed itself needs 30 rice per batch and
  Farmland only grows 10 rice every 750s. Hand-computed the true bottleneck: Farmland's rice
  output caps rice_processed at one batch every ~2250s, not every ~16s — the old code was
  overstating this chain's achievable rate by roughly **140x**. This bug already existed in the
  reference repo too (confirmed via its structurally identical `caramel_nut_chips` chain: nuts →
  caramel_nut_chips has the same gap), so it predates this session, but our new `find_coin_plan`
  surfaces it more readily than the original's single-best-item CLI mode did, since it actively
  searches every facility including deep chains.

**Fixed** with a new recursive `item_output_rate` function (`src/optimizer.rs`, just above
`calculate_item_requirements`): for a raw material, identical formula as before
(`yield_amount * facility_count / production_time` — zero behavior change, zero regression risk
for the common single-level case). For a processed item, the minimum of its own processing rate
and the recursively-computed rate of each of its own raw materials — so a 3+-level chain
correctly inherits the slowest link anywhere in the chain, not just the last step. Wired in with
a single, narrow change to the existing gathering-rate loop: only take the recursive path when
the immediate ingredient is itself processed (`raw.raw_materials.is_some()`), leaving the raw-
material case completely untouched.

**Verified live**: the isolated rice/Carousel Mill/Bouncy Brew Keg scenario now correctly shows
Farmland *and* Carousel Mill both "Funnels into Bouncy Brew Keg (sweet_rice_drink)" — both
upstream stages recognized as dedicated to feeding the chain, with the target reached in a
realistic 33m43s (a mix of this slow chain plus several faster single-stage facilities running in
parallel) rather than the old inflated estimate. The user's full saved scenario (sugarcane→
rock_candy, lemon→lemon_incense — both single-level chains) is **numerically unchanged**
(3h13m54s, identical to before this fix), confirming the fix only affects multi-level chains and
doesn't disturb anything already correct.

28/28 tests pass, clean WASM build, verified live against both the isolated repro and the user's
full saved scenario.

### 35. Bugfix: greedy selection could pick a combined item that was worse than selling separately

User shared a live screenshot and asked "does this seem correct to you?" — pottery (Crafting
Table: clay + scales) was claiming both Mineral Pile and Grass Blossom Mat. Hand-verified against
the actual CSV data: pottery's rate (~0.877 coins/sec at the shown facility counts) barely edged
out clay's own standalone rate (~0.868 coins/sec) — enough to sort ahead of it in the greedy
list — but clay's rate *plus* scales' own standalone rate (~0.322 coins/sec) sold separately
totals ~1.19 coins/sec, comfortably beating pottery. The greedy loop in `find_coin_plan` was only
ever comparing a candidate item's rate against *other individual items*, never against the sum of
what the facilities it would claim could earn independently — so it happily claimed two
facilities for a combined item worth less than what they'd have made apart.

**Verified the magnitude directly**: replayed the user's exact facility config twice — once with
Crafting Table at the level needed for pottery, once with it one level lower (forcing Mineral
Pile/Grass Blossom Mat to sell directly) — same coin target, same everything else. Pottery route:
1h36m51s. Direct-sell route: 1h34m2s. Confirms pottery was a real, reproducible ~3% loss, not
just a close call in theory.

**Fixed**: before the greedy loop, compute `standalone_rate[facility]` — the best rate each
facility can earn completely on its own (a raw material sold directly, `raw_materials.is_none()`,
zero dependency on anything else). When the loop considers a *processed* candidate item, it now
sums `standalone_rate` across every facility that item's `all_facilities` would claim, and only
selects the item if its own rate beats that sum. Raw items (which are themselves always some
facility's standalone option) are unaffected — the check only applies to processed/multi-facility
candidates. Verified live: the same scenario that showed pottery before now correctly shows
Mineral Pile → clay and Grass Blossom Mat → quick_scales both "Sells directly", reaching the coin
target in the faster 1h34m2s. The user's original saved scenario (sugarcane→rock_candy, lemon→
lemon_incense, both of which genuinely do beat their facilities' standalone rates) is unchanged —
confirms the fix only rejects combinations that are actually worse, not combinations in general.

28/28 tests pass, clean WASM build, verified live against both the pottery repro and the
unaffected saved scenario.

### 36. Feature: split facility capacity between a recipe and direct sale

User pushback on section 35's fix, and rightly so: "each clay sells for 34, each scale for 28,
pottery sells for 916 — pottery is way better, no?" Per-batch, yes — 916 beats 10×34 + 12×28 = 676.
Section 35 was comparing *rates*, not per-batch value, and concluded pottery was worse at the
user's facility counts (4 Mineral Pile : 1 Grass Blossom Mat) because 1 Grass Blossom Mat bottle-
necks pottery's throughput so much that 3 of the 4 Mineral Piles effectively sit idle waiting on
scales, wasting most of their clay-growing capacity. But that framing was still all-or-nothing —
the actual right answer, per the user's own follow-up question, is to split: dedicate only as
much Mineral Pile capacity as pottery's scales-bottleneck can use, and let the rest keep selling
clay directly.

**Hand-verified the value first** before implementing: with 4 Mineral Pile / 1 Grass Blossom Mat,
pottery needs ~0.0096 clay/sec but 4 Mineral Piles produce ~0.0255 clay/sec — only ~1.5 of the 4
are needed. Splitting gives pottery's income (0.877 coins/sec, unchanged, still scales-
bottlenecked) *plus* the leftover ~62% of Mineral Pile selling clay directly (~0.542 coins/sec) =
~1.419 coins/sec, beating both the all-pottery (0.877) and all-direct-sell (1.189) options from
section 35. Worked out the general case algebraically: whenever a recipe's sell value exceeds the
sum of (required_amount × sell_value) for its ingredients — i.e. there's a genuine "crafting
premium," true of basically every recipe worth crafting at all — claiming the recipe's bottleneck
ingredient *and* capturing 100% of any non-bottleneck ingredient's leftover as direct sale is
mathematically guaranteed to be at least as good as selling everything separately, regardless of
facility-count ratios. This meant section 35's rejection check (`eff.rate <= combined_standalone`)
needed to become leftover-aware, or it would now wrongly reject cases like this.

**Implemented** in `find_coin_plan` (`src/optimizer.rs`):
- New `resolve_ingredient` helper (extracted, shared substitution-lookup logic previously only
  inlined in `calculate_efficiencies`/`item_output_rate`).
- For each processed candidate item during the greedy pass, before deciding whether to select it:
  for each of its DIRECT raw-material ingredients that is itself a raw item (not a deeper
  processed chain) and doesn't share a facility with another of the same recipe's ingredients,
  compute that ingredient facility's consumption rate (at the item's own bottleneck
  batches/sec) vs its full solo capacity rate. Any shortfall is "leftover," valued at the
  facility's own best standalone rate (`standalone_item`, renamed from `standalone_rate` to also
  keep the winning item's identity, not just its number).
- The `combined_standalone` accept/reject check from section 35 now compares
  `eff.rate + leftover_value` (not just `eff.rate`) against the sum of standalone alternatives —
  so pottery, with its leftover now counted, correctly passes.
- Every income stream — selected items *and* leftover-capacity contributions — feeds into the
  same `(rate, lead_time)` list that already drove the binary search and timeline chart (sections
  32–33), so total_time/coins_produced/the coin-income graph all automatically account for
  leftover capacity with no separate bookkeeping.
- `coin_items` reporting: a facility with leftover shows a combined reason like `"38% funnels
  into Crafting Table (pottery); remaining 62% sold directly"` (or `"... sold directly as {item}"`
  if the facility's best standalone use differs from what the recipe needs — physically possible
  since leftover capacity is valued at whatever that facility's OWN best item is, not
  necessarily the same one feeding the recipe).

**Deliberately not handled** (scoped out, noted in code comments): ingredients that are
themselves processed (deep chains — sweet_rice_drink's rice_processed stays full-commitment, no
leftover credited even if Carousel Mill has slack), and recipes pulling multiple raw materials
from the SAME facility (e.g. dried_flowers' lavender + rose, both Farmland — splitting a single
facility optimally across two different ingredient ratios is a separate allocation problem,
already handled elsewhere for a different purpose by `calculate_optimal_allocation`, not
integrated with leftover accounting here). Both are rarer in practice than the direct single-
ingredient-per-facility case this targets.

**Verified live** against the user's exact scenario: Mineral Pile now shows "38% funnels into
Crafting Table (pottery); remaining 62% sold directly" (38% matches the hand-calculated
consumption fraction almost exactly), and total time improved to 1h32m12s — better than *both*
pure strategies compared in section 35 (1h34m2s direct-sell-only, 1h36m51s all-pottery),
confirming the split genuinely captures value neither pure option did. The original saved
scenario (sugarcane→rock_candy, lemon→lemon_incense) is numerically unchanged, since Farmland/
Woodland have no meaningful leftover at those facility counts — confirms the feature only adds
value where there's genuinely idle capacity to capture, without disturbing anything else.

28/28 tests pass, clean WASM build, verified live against the pottery split and the unaffected
saved scenario.

### 37. Replaced the coin-income graph with a per-product breakdown table

User ask: "Remove the result graph. Instead, I want a breakdown of each product produced. How
much, total worth, coins/sec." Straightforward swap — the timeline chart (section 33) answered
"when does income arrive," this answers "what's actually being made."

**Rust**: `CoinPlan::timeline: Vec<(f64, f64)>` replaced with `CoinPlan::products:
Vec<CoinPlanProduct>` (new struct in `models.rs`): `item_name`, `facility`, `rate_per_second`,
`units_per_second`, `lead_time`, `total_units`, `total_coins`. In `find_coin_plan`
(`src/optimizer.rs`), the `rate_lead_pairs: Vec<(f64, f64)>` used internally for the binary
search became `income_streams: Vec<CoinPlanProduct>` — richer up front (item identity +
units/sec, via a new `units_per_second_of` closure: `(rate / net_profit_per_batch) *
yield_amount`, which works uniformly for both raw and processed items) with `total_units`/
`total_coins` left at 0 until `total_time` is known, then filled in in a final pass and sorted by
`total_coins` descending. The checkpoint/timeline-building code is gone entirely — no longer
needed now that nothing renders a curve.

One entry per income stream, which — since section 36 — includes leftover-capacity portions
separately from what they feed: in the pottery example, "clay" (Mineral Pile, the 62% leftover
portion sold directly) and "pottery" (Crafting Table) show as two distinct rows with their own
rate/worth, not blended into one. Rows where `total_coins` ends up 0 (claimed but never actually
gets past its own lead time before the target is reached) are filtered out rather than shown as a
confusing zero.

**Web UI**: `web/index.html` — replaced the `#timeline-section` block with a
`#product-breakdown-section` table (Item | Facility | Amount | Coins/sec | Total Worth), placed
where the chart used to be, above "What Each Facility Should Do" (kept as-is, still answers a
different question — where things happen, not what's made). `web/app.js` — `renderTimeline`
replaced with `renderProductBreakdown`, which just renders `result.products` directly (already
sorted by the solver, no client-side computation). `web/style.css` — removed the now fully-unused
`.timeline-*` classes.

**Verified live**: for the pottery scenario, the table shows 5 rows (dried_strawberry, lemon_
incense, pottery, clay, wool) whose `total_coins` sum to exactly 28,726 — matching `coins_produced`
to the cent, confirming the breakdown isn't losing or double-counting anything relative to the
top-line number. "Nothing needed" edge case correctly hides the section.

28/28 tests pass, clean WASM build.

### 37b. UX fix: Amount column showed fractional items ("29.48 dried_strawberry")

User caught it immediately after section 37 shipped: "we can't craft .48 of a dried strawberry."
Correct — `total_units` is a continuous-rate estimate (same steady-state approximation used
throughout this calculator), fine as a rate but not as a "how many do I have" quantity a human
can act on. Fixed in `web/app.js`'s `renderProductBreakdown`: the Amount column now floors to a
whole number (`Math.floor(p.total_units)`) — the fractional remainder represents a batch still
in progress at the moment the coin target is reached, not a partial item you'd actually receive.
Rate (coins/sec) and Total Worth are left as continuous decimals, since those are legitimately
rates/aggregates, not physical item counts. Display-only change — `total_units` itself is
unchanged in the Rust/WASM layer. Verified live: dried_strawberry 29.48 → 29, pottery 4.61 → 4,
etc.

### 37c. Bugfix: Total Worth didn't reconcile with Amount × sell price

User caught it immediately again: "29 dried strawberries would be 13,920 coins... not that" (the
table showed 13,442.76). Correct catch — `total_coins` is *net profit* (sell price minus
ingredient cost, e.g. dried_strawberry's 480 sell value minus 24 in strawberry cost = 456/unit),
computed from the *unrounded* 29.48 units — not `sell_value * 29`, which is what "Total Worth"
actually implies to a reader doing the math by hand. Two separate problems layered on each other:
the column was measuring the wrong thing (profit, not worth), and even as profit it didn't match
the now-floored Amount column from section 37b.

**Fixed**: added `sell_value` to `CoinPlanProduct`/`JsCoinPlanProduct` (the item's per-unit price,
already available as `eff.item.sell_value` — trivial to expose, no new computation). In
`web/app.js`'s `renderProductBreakdown`, "Total Worth" is now computed client-side as
`Math.floor(total_units) * sell_value` — gross revenue on the exact whole-number amount shown,
so a reader can verify it by hand-multiplying the two visible columns and always get the third.
Relabeled "Coins/sec" → "Profit/sec" or clarity, since it's deliberately a *different* (net)
figure from Total Worth (gross) — added a line to the section's hint text explaining the two
won't relate by simple division, to head off a follow-up "why doesn't Profit/sec × time equal
Total Worth" question.

Verified live: dried_strawberry 29 × 480 = 13,920 (was 13,442.76); lemon_incense 23 × 345 = 7,935;
pottery 4 × 916 = 3,664; clay 77 × 34 = 2,618; wool 12 × 53 = 636 — all five rows now reconcile
exactly by hand.

### 38. Bugfix: setting a facility count to 0 silently reset it to the default

User (testing on a friend's behalf): "it thinks we have a bouncy brew keg and crafting table when
we don't. Those are 0." Real bug, and a familiar shape — `getInputValues()` in `web/app.js` built
each facility's payload with `parseInt(...).value) || f.defaultCount`. In JavaScript, `0 || x`
evaluates to `x`, since `0` is falsy — so entering `0` in the Count field didn't send `0` to the
solver, it silently sent back `f.defaultCount` (1, for most facilities: Farmland, Woodland,
Mineral Pile, Carousel Mill, Jukebox Dryer, Claw Game Cooker, Crafting Table, Phonolfactory
Table, Bouncy Brew Keg). Any facility a user doesn't actually own — as long as its default
happens to be 1 — was invisibly treated as "I own 1 of these" instead of "I own zero." This is the
exact same bug shape fixed earlier this session for the Resource Exchange "remaining today"
fields (`numberOrDefault`, since removed along with the rest of the Exchange feature in section
30) — that fix never got extended to facility count/level parsing, which still used the old
unsafe `||` pattern.

**Fixed**: restored a `numberOrDefault(value, fallback)` helper (parseInt-based, NaN-safe — falls
back only on blank/invalid input, not on a legitimate `0`) and a parallel `floatOrDefault` for the
coin target/current fields (kept `parseFloat` there instead of `parseInt`, since those didn't
actually have this bug — their fallback is already 0 — but switched them too for consistency and
to avoid truncating any future decimal input). Applied `numberOrDefault` to every facility's
`count` and `level`, and to all four module levels.

Verified live: set Bouncy Brew Keg and Crafting Table counts to 0 — both facilities now correctly
disappear from "What Each Facility Should Do" entirely (matching the existing `count > 0` filter
in `find_coin_plan`), and Grass Blossom Mat correctly falls back to selling quick_scales directly
instead of funneling into pottery, since Crafting Table (0 owned) can no longer make it. 28/28
Rust tests still pass (this was a JS-only fix, no Rust/WASM rebuild needed).

### 39. Feature: Wood Blocks/Mineral Sand reported as an informational "bonus"

User ask: "In the results, report the byproduct of wood blocks and mineral dust as a little
bonus." These were dropped as *optimization targets* in section 30 (trivially obtained by
expanding plots), but Woodland/Mineral Pile items still carry a `byproduct` field in the data
model that's gone completely unused since then — this surfaces it again, purely informationally.

**Key correctness point**: a byproduct comes from *growing* the item, not from *selling* it — so
it applies whether that facility's output is sold directly, fully funneled into a recipe (e.g.
Woodland's lemon feeding Phonolfactory Table's lemon_incense still yields Wood Blocks, even
though no lemon is ever sold on its own), or split between the two (section 36). Implemented as a
post-processing pass in `find_coin_plan` (`src/optimizer.rs`), after `total_time` is known: walks
every claimed facility and credits `byproduct_amount * facility_count / production_time`, scaled
by whatever fraction of that facility is growing the byproduct-bearing item (100% for a direct
sale or full funnel, split proportionally via the existing `facility_leftover` map otherwise),
active only after that item's own lead time has passed — same accounting pattern as everything
else in this function. New `facility_feeding_item: HashMap<&str, &ProductionItem>` tracks which
raw item each ingredient-supplier facility is actually growing (populated alongside
`facility_leftover`, but for *every* resolved ingredient, not just ones with leftover capacity —
needed because a fully-consumed facility, like Woodland feeding lemon_incense with no spare
capacity, still needs its byproduct credited even though it never appears in the leftover map).

New `CoinPlan::byproducts: Vec<(String, f64)>` (only non-zero resources included) → `wasm.rs`'s
`JsCoinPlanResult.byproducts`, serializing as `[[name, amount], ...]`. **Web UI**: new
`.byproduct-note` box (amber border, matching the existing `.energy-info`/`.parallel-info`
callout convention) between the summary tiles and the Product Breakdown table — `web/app.js`'s
`renderByproducts` reads `result.byproducts`, floors each to a whole number (same reasoning as
Product Breakdown's Amount column — can't receive a fractional Wood Block), and renders something
like "**Bonus** You'll also pick up 4,421 Wood Blocks and 3,682 Mineral Sand along the way."
Hidden entirely when nothing byproduct-bearing is in the plan.

28/28 Rust tests pass, clean compile. **Verified live** (section 40): with 5 Mineral Pile
(level 3) selling quartz, hand math predicts 172 Mineral Sand over the plan's total time — matches
the live UI exactly. The standalone `.byproduct-note` box described above was subsequently
replaced by table rows at the bottom of Product Breakdown (section 40); the underlying
`CoinPlan::byproducts` computation described here is unchanged.

### 40. Byproduct rows moved into Product Breakdown; fixed mislabeled intermediate facilities;
Jukebox Dryer "no profitable item" confirmed correct

Three user asks in one pass:

**1. Byproducts into the table.** User wanted Wood Blocks/Mineral Sand moved from the standalone
callout box (section 39) to the bottom of the Product Breakdown table, visually distinguished.
`web/app.js`'s `renderProductBreakdown` now appends one row per byproduct after the normal
product rows, class `byproduct-row` (amber/italic via `#product-breakdown-table tbody
tr.byproduct-row` in `style.css`, dashed top border to separate it from sold items) with `—` in
the Facility/Profit-sec columns and "not sold" in Total Worth, since these aren't sold for coins.
The old `.byproduct-note` div and `renderByproducts` function were removed entirely.

**2. Mislabeled intermediate facilities.** User: "Bouncy Brew Keg and Carousel Mill are saying
they are producing soybean when they in fact are USING soybean to produce other thing." Real bug
in `find_coin_plan`'s `coin_items` construction (`src/optimizer.rs`): for a facility supplying a
multi-level chain, the code fell back to `eff.requires_raw`, which only lists ROOT raw materials
(e.g. "soybean"), never the names of intermediate processed items (e.g. "soy_sauce" made at
Bouncy Brew Keg from that soybean). Every facility in the chain that wasn't the root grower AND
wasn't the final seller got mislabeled with the root material's name. Fixed by checking
`eff.intermediate_steps` (already recursively collected by `calculate_efficiencies`, just
previously unused here) first — it has the correct `(item_name, facility, required_amount)` per
processing step — and only falling back to the root-material logic for facilities that are
genuinely growing a raw material rather than processing one. Verified live with a
sugarcane→rock_candy (Claw Game Cooker)→potato_kvass (Bouncy Brew Keg) chain: Claw Game Cooker
now correctly shows "rock_candy / Funnels into Bouncy Brew Keg (potato_kvass)" instead of
"sugarcane+potatoes". 28/28 tests still pass.

**3. Jukebox Dryer "no profitable item" — investigated, not a bug.** User suspected
shredded_coconut (5 coconut → 900 coins) should beat selling the coconut raw. Hand math initially
suggested the same (~6.1 coins/sec funneling all Woodland output into shredded_coconut vs. 3.99
coins/sec selling quick_coconut directly) — until checking the actual blocker: shredded_coconut
requires Jukebox Dryer facility level 3, and the user's is level 2. It's filtered out before
profitability is even evaluated (`calculate_efficiencies`'s facility-level check), and no other
Jukebox Dryer item at level ≤2 has an unclaimed raw-material facility to draw from (Farmland's
committed elsewhere, Woodland's committed elsewhere) — so "no profitable item currently available"
is the correct output, just for a level-gating reason rather than a profitability one. No code
change.

### 41. Fixed: greedy solver's sort order could permanently lock a better combo out

User: pottery scenario (5 Mineral Pile level 2, 1 Grass Blossom Mat level 1, 1 Crafting Table
level 2, mineral_detector 2) — Crafting Table sat idle selling nothing while Mineral Pile and
Grass Blossom Mat sold clay and quick_scales directly, even though pottery (clay + scales) should
have been worth crafting.

Real bug, and a different one from sections 35/36's `combined_standalone` check (which correctly
rejects a combo when it's *not* worth it — that logic was fine). This was the opposite failure:
the combo genuinely *was* worth it, but never got a chance to prove it. `find_coin_plan`'s greedy
loop sorts candidates by `effective_profit_per_second` — pottery's own rate (0.88/sec, gathering-
bottlenecked by Grass Blossom Mat's single facility) sorts *below* clay's standalone rate
(1.08/sec), so clay gets evaluated first, claims all 5 Mineral Pile with simple conflict-free
selection, and by the time pottery's turn comes around both its facilities are already occupied —
it never even reaches the `combined_standalone` check that would've compared it fairly. Confirmed
by hand: pottery's own rate (0.88) plus 70% leftover clay capacity handed back (0.7 × clay's 1.08
= 0.76) totals 1.64/sec, which comfortably beats clay (1.08) + quick_scales (0.32) = 1.40/sec sold
separately — pottery should win, but the *sort key* used to decide iteration order didn't include
that leftover value, only the recipe's own bottlenecked rate.

Fixed by splitting the leftover computation out of the greedy loop into its own pass that runs
over every candidate first (`leftover_cache: HashMap<&str, (f64, Vec<leftover>, Vec<feeding>)>`
in `src/optimizer.rs`), then sorting candidates for the greedy loop by `effective_profit_per_second
+ leftover_cache value` instead of `effective_profit_per_second` alone — so a combo's TRUE value
(including what it hands back) determines when it gets first pick, not just its own bottlenecked
rate. The loop body then looks up the cached leftover data instead of recomputing it inline (same
logic, now runs once per candidate instead of only for candidates actually reached).

Verified live: same scenario now shows Crafting Table producing pottery (31 units, 0.88 coins/sec,
28,396 total worth — 31 × 916 reconciles exactly), Grass Blossom Mat's quick_scales funneling in
at 100%, and Mineral Pile split 30% pottery / 70% direct clay sale — matching the hand math above
almost exactly (predicted 0.88/sec and a ~30/70 split). Total time improved from 9h48m50s to
9h41m49s for the same 600,000-coin target, as expected for a strictly better plan. 28/28 tests
still pass.

### 42. Joy Wheel Loom item data added; facility inputs grouped by category

User: "The Joy Wheel Loom does not appear to be part of the app. I thought I gave you that info?"
Checked the transcript — Joy Wheel Loom had only ever been *named* in the original facility list
(section 11), never given item-level data; BETA_NOTES had been correctly flagging it "Zero data
yet" the whole time. Not a bug, just never populated. User then provided the recipe table:

| Item | Level | Ingredients | Workload | Duration | Value |
|---|---|---|---|---|---|
| cotton_thread | 1 | 6 cotton | 23 | 18s | 213 |
| woolen_yarn | 1 | 4 wool | 23 | 18s | 287 |
| palm_rope | 2 | 4 palm | 23 | 18s | 779 |
| cotton_fabric | 3 | 1 cotton_thread | 27 | 18s | 723 |
| wool_fabric | 3 | 1 woolen_yarn | 27 | 18s | 797 |
| palm_fabric | 4 | 1 palm_rope | 27 | 18s | 1604 |

Added as `data/joy_wheel_loom.csv`, same `load_processing_no_energy` shape as Crafting Table
(Energy column was "N/A" for every row, unlike Carousel Mill/Jukebox Dryer/Claw Game Cooker which
track energy). Wired into both loaders: `src/data.rs::load_all_data` (native/CLI) and
`src/wasm.rs`'s hand-duplicated `include_str!`-based loader (the WASM build can't read files off
disk at runtime, so it embeds each CSV at compile time in its own copy of this same loading logic
— easy to forget when adding a facility, since `cargo test` alone won't catch a WASM-only miss).
Ingredients resolve to existing raw items: "cotton" (Farmland, level 2), "wool" (Nimbus Bed),
"palm" (Woodland, level 3, matches the table's "Palm Bark").

Verified live: with 5 Woodland + 1 Joy Wheel Loom (level 4), palm_fabric shows 4 units at 3.48
coins/sec, 6,416 total worth. Hand math for the full 3-level chain (Woodland's palm supply →
Joy Wheel Loom's palm_rope → Joy Wheel Loom's palm_fabric, bottlenecked throughout by palm's own
growth rate thanks to the `item_output_rate` recursion from section 34) predicts 1565 coins/batch
net profit × 0.0022222 batches/sec = 3.478 coins/sec, and 4.81 units over the plan's active
window (floors to 4, 4 × 1604 = 6,416) — matches exactly. Facility plan correctly shows Woodland
funneling "palm" (not the intermediate "palm_rope") into Joy Wheel Loom, confirming section 40's
intermediate-facility-labeling fix holds for this new facility too.

**Second ask, same message:** group the facility input cards by category (Materials / Aniimo
Materials / Materials Processing — Auxiliary Facilities excluded, they don't produce items, see
section 11), matching the user-provided facility list. Added a `category` field to each entry in
`FACILITIES` (`web/app.js`) and a `FACILITY_CATEGORIES` display-order array; `renderFacilityCards`
now groups cards into a labeled section per category instead of one flat grid. `index.html`'s
`#facilities-grid` div was repurposed from the card grid itself to an outer container (renamed
`.facilities-container` in CSS, flex column) holding one `.facility-category` block per category,
each with its own `.facility-category-title` heading and inner `.facilities-grid` (unchanged
2-column card grid). Verified live: three sections render in the requested order, each still a
2-column grid.

### 43. Two related rate-correctness bugs: shared-resource double-counting, and processing
facilities with unexplained idle capacity

User: "if I say I have 2 Carousel Mills, I would expect 1 to be assigned tofu, and the other
coconut oil... Currently it's allocating both to tofu... what do you think?" Investigating this
surfaced a bigger, previously-undetected bug underneath it.

**Bug 1 — shared raw-material double-counting.** soy_sauce_tofu needs both soy_sauce (Bouncy Brew
Keg) and tofu (Carousel Mill), and both independently need soybean from Farmland. The old rate
calculation computed each ingredient's achievable rate via `item_output_rate` in isolation — each
call assumed it alone had exclusive access to all 20 Farmland's soybean output. In reality the two
branches have to share one soybean supply. Hand-verified the live bug before fixing: with 20
Farmland, soy_sauce_tofu displayed 12.24 coins/sec; correct combined-sharing math gives ~6.1 —
almost exactly double.

**Fix:** replaced the old per-ingredient "compute each branch's rate independently, take the min"
approach with a whole-tree walk (`accumulate_demand`/`compute_resource_demand`/`batch_rate_bound`
in `src/optimizer.rs`) that accumulates, per FACILITY touched anywhere in an item's ingredient
tree, total *utilization* — batches/sec of whatever runs there, weighted by that item's own
`production_time`, required per one batch/sec of the tree's root. Utilization is additive by
construction, so it correctly handles both ways a facility can be shared: the same item needed via
two branches (soybean, above) and two DIFFERENT items time-sharing one facility for the same chain
(discovered while verifying: Claw Game Cooker both turning sugarcane into rock_candy AND
assembling rock_candy+tofu into tofu_cake — the OLD code independently checked "does Claw Game
Cooker have capacity for the assembly step" and "does it have capacity for rock_candy" without
ever adding them together, silently allowing more throughput than one facility can actually give;
this was a pre-existing bug, not something introduced by this session, just uncovered by building
this fix). `item_output_rate` (the old per-branch function) was deleted entirely — its one caller
and its own recursive self-call were both replaced by `compute_batch_rate`'s successor
`batch_rate_bound`. New `ProductionEfficiency.facility_demand: Vec<(String, f64, Vec<String>)>`
(facility, utilization, item names hosted there) is computed once in `calculate_efficiencies` and
reused both for `effective_profit_per_second` and for Bug 2 below.

**Bug 2 — idle processing capacity never reallocated.** The user's literal report: Carousel Mill
capacity beyond what its assigned item needs (because the true bottleneck is upstream, e.g.
soybean supply) just sits idle — the existing "leftover capacity" mechanism (section 36) only
covered ingredient-SUPPLIER facilities (Mineral Pile growing more clay than a recipe consumes),
never intermediate PROCESSING facilities. Two owned Carousel Mills don't help tofu's rate at all
if soybean already caps it at what one mill could do.

**Fix:** a new pass in `find_coin_plan`'s selection loop, run right after a chain is accepted,
walks `eff.facility_demand` for every facility whose hosted items include at least one processed
item (raw growers are explicitly excluded — they stay on the existing byproduct-aware leftover
path, since this new pass doesn't track `facility_feeding_item` and crediting a raw grower here
would silently under-count its Wood Blocks/Mineral Sand byproduct). For each, compares actual
utilization at the chain's achieved rate against owned capacity; any leftover is credited to that
facility's own best OTHER item, subject to that alternative's OTHER required facilities not
already being claimed by something else (checked against `occupied` as of right now, not
precomputed before sorting like the section-36 leftover, since a processed alternative's
feasibility genuinely depends on what's already been claimed). Reuses the existing
`facility_leftover` map and its "X% funnels into Y; remaining Z% sold as W" rendering — extended
the two `coin_items` match arms that didn't already check it (the facility's-own-item "sells
directly" case, and the intermediate-processing-step case) to match section 36's raw-material-
supplier arm.

**A bug caught during verification, not before shipping:** the first version of this fix marked
`alt`'s other required facilities as `occupied` but never updated `claimed_by` for them — so a
facility silently feeding a credited leftover (e.g. Farmland growing cotton for a credited
cotton_fabric) showed "No profitable item currently available" in the plan table despite actually
being in use. Fixed by also inserting `claimed_by.insert(f, alt)` for each of `alt`'s OTHER
facilities (not the shared one itself, which correctly keeps pointing at the primary chain so its
own row's split rendering stays intact).

**Verified live**, twice, both reconciling exactly by hand:
- 20 Farmland/10 Woodland/2 Carousel Mill/1 Jukebox Dryer/1 Claw Game Cooker: strawberry_cream_puff
  (coconut_oil + dried_strawberry) displayed 11.29 coins/sec; hand math (Woodland-coconut-limited
  at 0.007111 batches/sec × 1587 net profit/batch) gives 11.286 — matches. Carousel Mill correctly
  shows no leftover split here, because its only alternative items all need Farmland, which is
  already claimed — confirms the "no eligible alternative" rejection path works too.
- Same setup with Jukebox Dryer removed (forcing a different winner): Joy Wheel Loom split 15%
  palm_fabric / 85% cotton_fabric. Hand math: palm_fabric 6.956 coins/sec (Woodland-palm-limited),
  leftover fraction 84.5%, cotton_fabric's own full rate 6.347 coins/sec × 0.8452 = 5.365 — all
  three match the displayed 6.96 / 85% / 5.36 exactly. Farmland's row correctly attributes to
  "Funnels into Joy Wheel Loom (cotton_fabric)" after the claimed_by fix, instead of the
  pre-fix-verification "No profitable item currently available".

28/28 tests pass throughout (structural changes only — no test asserted on `item_output_rate`'s
exact old numbers directly, so none needed updating).

### 44. `find_coin_plan` rewritten as a linear program — replaces greedy + three leftover patches

User: "What would it take to make our calculations completely exhaustive?" — after being told
plainly (in response to "how confident can I be") that the greedy solver is not a proof of
optimality, just the best result a particular search order happened to find. Followed by "Yes" to
a proposed implementation plan (see the plan file this session referenced, and section 43's
context for why greedy-plus-patches had become the wrong shape for the problem).

**The model:** one LP variable per candidate item (`calculate_efficiencies`'s output, unchanged)
= its batches/sec; objective maximizes `Σ net_profit_per_batch_i × x_i`; one constraint per owned
facility: `Σ utilization_i,f × x_i ≤ facility_count_f`, where `utilization_i,f` comes straight out
of `ProductionEfficiency.facility_demand` (built for section 43's Carousel Mill fix — this session
already had the exact coefficients an LP needs before ever deciding to build one). Solved with
`microlp` (pure Rust, verified to compile cleanly under `wasm-pack build --target web` before any
integration code was written — the wasm bundle grew from ~198KB to ~352KB, still loads instantly).

**What this replaced:** `standalone_item`, `leftover_cache`, the sort-by-"true value" ordering
hack from section 41, the greedy claiming loop, and section 43's processing-facility leftover pass
— three separate mechanisms, each added to fix one more scenario the previous ones missed. All
subsumed by one LP solve: a facility ending up split between several items is just what the
solution looks like when that's optimal, not a bolted-on special case. `claimed_by` +
`facility_leftover` + `facility_feeding_item` were replaced by one `facility_usage: HashMap<&str,
Vec<(&ProductionEfficiency, f64)>>` (facility → every contributing item and its capacity
fraction), which naturally supports any number of contributors per facility instead of the old
hardcoded "one primary + at most one leftover" — both the `coin_items` "why" text and the
byproduct-crediting pass were generalized to loop over however many contributors there are.

**Bug caught by this rewrite, not introduced by it:** section 43's processing-leftover pass
explicitly *excluded* raw growers from its leftover check specifically because it couldn't track
byproduct attribution for them. Under the unified `facility_usage` model there's no separate
"which pass credited this" bookkeeping to get out of sync, so that restriction — and the
byproduct-undercounting risk it existed to avoid — both went away as a side effect.

**A display bug caught during live verification** (not before shipping, same pattern as section
43): two different chains sharing a facility can resolve to the same material name (clay sold
directly *and* clay feeding pottery both label as "clay"), which rendered as "clay + clay" before
deduplicating the label list.

**Verified live**, with the exact facility setup from the section 43 write-up (which had produced
strawberry_cream_puff alone under greedy): the LP now runs strawberry_cream_puff, soy_sauce_tofu,
*and* maple_candy_star simultaneously, splitting Claw Game Cooker three ways, Carousel Mill and
Woodland two ways, and Farmland two ways between strawberry_cream_puff and soy_sauce_tofu (both
independently drawing on the same 20 Farmland) — something the old greedy approach structurally
could not do, since it could only ever award a contested facility to one winning recipe. **Total
time for the same 600,000-coin target dropped from 12h59m43s to 11h32m43s** — a real, measurable
improvement, not just a cleaner implementation. Hand-verified two of the splits exactly: Claw Game
Cooker's three percentages (22%/3%/2%) reconcile precisely from each item's displayed rate and its
own `production_time`-derived utilization; Farmland's two consumers (strawberry_cream_puff's
dried_strawberry branch, utilization 2250, and soy_sauce_tofu's *combined* soy_sauce+tofu branches,
utilization 4500) sum to exactly 20.0 of its 20-unit capacity — precisely matching the displayed
78%/22% split.

Added 4 new tests to `tests/optimizer_tests.rs` — `find_coin_plan` had zero direct test coverage
before this (every fix this session was verified live by hand only): the shared-soybean scenario
from section 43 (asserts the corrected ~6.1 coins/sec range, not the ~12.2 doubled value), the
Joy Wheel Loom leftover-split scenario (asserts both items appear and Farmland's row correctly
attributes to the credited item — a regression test for the `claimed_by` bug from section 43), and
basic feasible/infeasible cases. All 32 tests (28 previous + 4 new) pass. `resolve_ingredient` was
deleted (fully superseded by `resolve_raw_material`, its only remaining caller went away with the
greedy loop). `find_best_production_path` and its allocation helpers (used by the CLI, not the web
app) were left untouched — confirmed via grep that `find_coin_plan` is `wasm.rs`'s only caller.

### 45. Facility-plan percentages replaced with actionable whole-unit counts

User, looking at the section 44 multi-way splits live: "Percentile usage of farmland, etc just
doesn't make sense for humans. One farmland makes one type of crop at a time. It needs to be in
terms of X number farmland for Y crop, etc." Correct — the LP's fractional capacity shares are the
right math for computing rates, but a player assigns a *whole plot* to a crop, not a percentage of
one; "78% funnels into X" isn't something anyone can act on.

Added `apportion_counts` (`src/optimizer.rs`, largest-remainder method — the same apportionment
technique used to allocate parliament seats) to convert each facility's contributor fractions into
whole counts that sum as close as possible to the true split, e.g. Farmland's 78%/22% becomes "16
strawberry + 4 soybean" instead of two percentages. Two cases can't be expressed as whole units and
fall back to a "X% of the time" phrasing instead: a facility with only 1 owned unit (it genuinely
alternates between recipes as upstream inputs allow, not a discrete split), and a facility where
every contributor's true share is too small to round up to even one whole unit despite real
production happening — caught live on Carousel Mill (6%+1% of 2 units), where the first version of
this fix collapsed straight to "2 idle," silently hiding that real coin value was still being
produced from a sliver of its capacity.

Verified live against the exact section-44 scenario: Farmland → "16 strawberry + 4 soybean"
(matches the hand-verified 78%/22% split rounded), Mineral Pile → "4 clay + 1 clay" (matches the
70%/30% split), Woodland → "10 quick_coconut" (maple_candy_star's 2% share rounds to 0 and is
correctly dropped, since you can't carve out a fraction of one plot for it), and Claw Game
Cooker/Carousel Mill correctly keep percentage phrasing (1 owned unit; too-small-to-round-up,
respectively) rather than misrepresenting either case as a clean whole-number split. All 32 tests
still pass.

### 46. Section 45's rounding made authoritative — Total Time/Coins Produced now derived FROM the
integer facility counts, not alongside them

User: "Don't just round for the sake of the user. The math needs to be correct from start to
finish." Correct, and section 45 didn't do that — it rounded facility percentages into whole
counts purely for *display*, while Total Time, Coins Produced, and the Product Breakdown stayed
computed from the continuous LP solution. If a player assigned exactly "16 Farmland to strawberry,
4 to soybean" as recommended, the real achievable rate could differ from what was reported, since
16/4 isn't precisely the LP's 15.625/4.375 split.

**The fix rests on a physical distinction, confirmed against `src/data.rs`'s loaders directly**:
`load_farmland`/`load_woodland`/`load_workload_raw_material`/`load_nimbus_bed` always set
`raw_materials: None`; `load_processing_*` always set it to `Some` — a clean, total split with no
facility ever mixing both. A GROWER facility (Farmland, Woodland, ...) commits each plot to one
crop for its whole cycle — genuinely can't split 78/22. A PROCESSOR facility (Claw Game Cooker,
Carousel Mill, ...) processes whatever ingredients are ready and can legitimately cycle between
recipes in whatever proportion their inputs allow — "22% of the time" is real, achievable
behavior there, not an approximation. Section 45 wrongly forced whole-unit rounding onto both
kinds, which is what produced last time's awkward "1 owned unit" and "too small to round" special
cases — both symptoms of applying an integer constraint where none actually exists.

**Key insight that keeps the fix small**: the continuous LP's per-item rate already correctly
accounts for fair sharing at every PROCESSOR facility (that's the whole point of solving jointly).
The only thing integer rounding can do is reduce a GROWER facility's supply below what the
continuous relaxation assumed. So the only correction needed is `final_rate_i = min(continuous_
rate_i, min over grower facilities g touched by i of (assigned_count_i,g / utilization_i,g))` —
capping the already-correct continuous rate, never independently re-deriving the processor-shared
portion (which would silently reintroduce the exact shared-resource double-counting bug from
section 43/44 — traced through what redoing Claw Game Cooker's three-way split independently per
item would do: each item would assume exclusive access to it again).

New `src/optimizer.rs` functions: `is_grower_facility` (the structural check above),
`build_grower_assignment` (apportions each grower facility's continuous shares into authoritative
whole counts via the existing `apportion_counts`), `final_rate_for` (the `min()` formula). Every
downstream number — `income_streams`, the lead-time binary search for `total_time`/
`coins_produced`, the Product Breakdown, byproduct crediting — now consumes `final_rate_for`'s
output instead of the continuous allocation directly, so everything reported is derived from the
same whole-unit facts as what's displayed for growers. `coin_items` now branches cleanly on
`is_grower_facility`: growers read `grower_assignment` directly (exact integers, no re-
apportioning), processors always get percentage/time-share phrasing regardless of owned count —
removing the previous "1 owned unit or too-small-to-round" heuristic entirely, since that was
only ever needed because processors were being rounded when they never should have been.

**A second real bug, caught live before calling this done**: a chain needing two or more DIFFERENT
grower facilities (maple_candy_star needs both Woodland's maple_syrup and Starfall Hammock's
star) can get "stranded" — each grower is apportioned independently, so it's possible for one to
round that chain's share to zero (Woodland gave all 10 plots to the far-more-valuable
quick_coconut, leaving maple_candy_star's maple_syrup share at 0) while the OTHER grower
(Starfall Hammock) still showed a whole-unit assignment to a chain that could now never actually
produce anything — recommending a facility be dedicated to nothing. Fixed with an iterative
exclusion loop in `find_coin_plan`: after computing grower assignments, check whether any chain's
final rate is 0 despite a nonzero grower assignment; if so, exclude it and re-solve the whole LP
from scratch without it, repeating until stable (converges quickly — each pass excludes at least
one more item from a small candidate set). This lets the LP find Starfall Hammock's genuinely
useful fallback (selling star directly) instead of a patch that would've just marked it idle.

**Verified live** against the same 20 Farmland / 10 Woodland / 2 Carousel Mill / 1 Claw Game
Cooker scenario, hand-reconciling Total Time/rates directly from the displayed integers this time
(the check section 45 was missing): strawberry_cream_puff's displayed 11.29 coins/sec matches
1587 × min(16/2250, 10/1406.25) = 1587 × 0.0071111 = 11.2854 → rounds to 11.29 exactly; soy_sauce_
tofu's displayed 1.22 matches 1377 × (4/4500) = 1.2242 → rounds to 1.22 exactly. Both Total Worth
figures reconcile exactly against their displayed amounts (295 × 1650 = 486,750; 36 × 1425 =
51,300). Starfall Hammock's row correctly changed from the stranded "Funnels into Claw Game
Cooker (maple_candy_star)" to "Sells directly" as star, with maple_candy_star correctly absent
from the Product Breakdown entirely. All 32 tests pass unmodified — the existing shared-soybean
test's rate assertion range comfortably tolerated the small, honest reduction from Farmland's
4-vs-4.375 rounding (1.22 vs the previous 1.34).

### 47. Facility-plan table simplified to single-product rows with whole-unit processor
dedication; a zero-capacity LP bug found and fixed; `find_coin_plan` split into
`find_production_plan` + `time_to_reach_goal`; Bud Tickets support restored

**Facility-plan table, round one — one row per product, grouped like the Facilities input
section.** User: "we are still overcomplicating the WHY... something like 2 Carousel Mill should
not double count and report '2' in separate rows... stop reporting percentages." Two fixes:

- Every `PlanStep` (renamed from `CoinPlanStep`) row is now exactly one facility producing exactly
  one item — a facility split three ways gets three rows instead of one row with a joined "16
  strawberry + 4 soybean" label or a semicolon-chained "22%: A; 3%: B" reason. New `PlanStepStatus::
  Idle` variant for a row representing leftover unused capacity, instead of appending "N idle" text
  onto whichever row happened to sort first.
- The web results table groups rows into the same three category sections (Materials / Aniimo
  Materials / Materials Processing) as the Facilities input cards, via the existing
  `FACILITY_CATEGORY_BY_NAME` lookup already built for the input side.
- **Processor facilities (Carousel Mill, Claw Game Cooker, ...) now get whole-unit dedication
  whenever it's valid**, instead of always reporting a raw time-share percentage. Key insight: a
  facility physically dedicated to one recipe can only ever achieve a rate *at or above* its share
  of a jointly-run one (a dedicated unit's throughput ceiling is a whole unit's worth, not a
  fraction), so — as long as every contributor's rounded-up need (`ceil(fraction)`) still fits
  within what's owned — dedicating whole units is strictly clearer and changes no rate/total
  computed elsewhere. Verified live: Carousel Mill (2 owned, coconut_oil + tofu) went from "2 |
  coconut_oil | 6% of the time..." / "2 | tofu | 1% of the time..." (reads like double-dedication)
  to "1 | coconut_oil | Used for strawberry_cream_puff" / "1 | tofu | Used for soy_sauce_tofu".
  Percentages only survive when genuine physical contention exists (e.g. Claw Game Cooker owning 1
  unit but hosting 2 recipes — `ceil` sum exceeds owned, so real time-sharing is unavoidable and
  the only honest description left). Also dropped the destination-facility name from every reason
  string ("Funnels into Claw Game Cooker (X)" → "Used for X") per the user's requested phrasing.

**A real correctness bug, found while building a test scenario for the user (not touched by any
work above).** Investigating why soy_sauce_tofu's rate compares to dried_strawberry's, I zeroed
Bouncy Brew Keg to isolate one branch — and soy_sauce_tofu kept producing at its full rate anyway,
confirmed by calling the wasm `optimize_coin_plan` function directly (bypassing the UI) with
`Bouncy Brew Keg: {count: 0}`. Root cause in `solve_facility_allocation`
([optimizer.rs:2057](../src/optimizer.rs)): the LP constraint loop **skipped adding a constraint
entirely** for any facility with 0 capacity (`if capacity <= 0.0 { continue; }`) instead of adding
a `≤ 0` constraint — so a facility you own none of was treated as having *unlimited* supply for any
recipe using it as an INTERMEDIATE step (a separate, existing check already caught the case where
an item's own ROOT facility is unowned, which is why this specific bug never showed up for
single-facility items). One-line fix: always add the constraint, even at capacity 0. Regression
test: `test_find_coin_plan_never_produces_via_unowned_intermediate_facility`.

**Reshaping the calculator's shape, per user request: "the calculator tells you the fastest way to
make the currency you want... in units of X/sec. Using that, you can then input a goal amount and
it flexibly tells you how long it will take."** Traced `find_coin_plan` and confirmed the split was
already latent in the code: everything through building `income_streams`/`coin_items` never reads
`target_coins`/`current_coins` — only the binary-search-for-time-to-target step at the end does.
Split into two functions:

- `find_production_plan(items, currency, facility_counts, module_levels) -> Option<ProductionPlan>`
  — the LP solve + facility-plan table, no goal needed. `ProductionPlan` carries a headline
  `rate_per_second` (sum of every income stream's rate) plus `income_streams`/`byproduct_rates`
  with totals left at 0.0 — target-independent, so it's exactly what the user asked to see first.
- `time_to_reach_goal(plan, target, current) -> Option<GoalResult>` — pure math over the
  already-solved plan (no facility-allocation re-solve at all), so it's cheap enough to call on
  every keystroke of a goal-amount field. Byproduct crediting had to change shape to support this:
  previously computed `rate * active_time` in one step using the not-yet-known `total_time`; now
  `find_production_plan` stores un-summed `(resource, rate, lead_time)` triples (`byproduct_rates`),
  and `time_to_reach_goal` sums them into totals once a plan's duration is known.
- `wasm.rs` mirrors this as two exported functions, `find_plan`/`time_to_reach` (was the single
  `optimize_coin_plan`). `JsProductionPlan` derives both `Serialize` and `Deserialize` since the JS
  caller holds the plan object returned by `find_plan` and passes it back byte-for-byte as part of
  the input to `time_to_reach` — no re-serialization logic needed on either side of the boundary.
- Web UI: Calculate now only computes the plan (facilities/currency/modules) — shows "Your Rate"
  and the facility-plan table immediately. A new "Set a Goal" panel below it updates Total
  Time/Amount Produced/Product Breakdown live on every `input` event on the Target/Current Amount
  fields, using the in-memory `lastPlan` object, confirmed via direct testing to require no extra
  network/wasm round-trip through the facility solver. Changing currency clears `lastPlan` and
  hides the goal panel until Calculate is pressed again, rather than show a stale plan for the
  wrong currency.

**Bud Tickets restored as an optimizable currency** — reverses the "deliberately excluded" decision
from an earlier pass (this file, "Bud Tickets deliberately excluded... modeling that shared-facility
tradeoff alongside coins is a bigger problem than what's needed right now"). The concern was joint
coins+Bud-Tickets optimization on shared facilities (Crafting Table, Claw Game Cooker); the actual
ask turned out to be simpler — a single **Coins or Bud Tickets** radio choice per calculation, which
`calculate_efficiencies`'s existing `target_currency` filter already handled correctly (confirmed by
a pre-existing, already-passing `test_calculate_efficiencies_bud_tickets` test) — `find_coin_plan`
was the only place hardcoding `"coins"`. New end-to-end test
(`test_find_production_plan_bud_tickets_end_to_end`) confirms `advanced_soy_sauce_tofu` appears
under `bud_tickets` and not `coins` for an otherwise-identical facility setup, and vice versa for
`soy_sauce_tofu`. Verified live: switching to Bud Tickets and recalculating showed an entirely
different item set (advanced_soy_sauce_tofu, advanced_flower_bread, premium_wood_sculpture) at
1.85 Bud Tickets/sec.

Struct renames throughout (`src/models.rs`, `src/optimizer.rs`, `src/wasm.rs`,
`tests/optimizer_tests.rs`) since "Coin"-prefixed names became actively misleading once Bud Tickets
is a real option: `CoinPlan` → `ProductionPlan`, `CoinPlanStep`/`CoinPlanStepStatus` → `PlanStep`/
`PlanStepStatus`, `CoinPlanProduct` → `PlanProduct` (`total_coins` field → `total_value`), new
`GoalResult` for the target-dependent half. `src/main.rs` (the CLI) uses the older single-item
`find_best_production_path` path entirely and was untouched by any of this.

All 26 tests pass (`cargo test`), `wasm-pack build` clean, verified live end-to-end.

### 48. Processor facilities can't be time-shared between recipes — "set and left" is the real
constraint, not a fractional split

User, on seeing "Claw Game Cooker | 1 | strawberry_cream_puff | 22% of the time: Sells directly":
"we should not be switching the Claw Game Cooker some fraction of the time. This is meant to be a
'set and leave it' kind of deal." Correct, and this was a real modeling bug, not just a wording
one — section 47's "genuine contention" percentage fallback documented that the continuous LP
relaxation could split a facility with more candidate recipes than owned units into a fractional
time-share, and `total_time`/`rate_per_second` were computed as if that split were achievable. It
isn't: a player assigns a facility to one recipe and it runs that recipe continuously.

**Key invariant that made the fix tractable**: for a facility used by only ONE item, that item's
`fraction` (utilization × rate / capacity) is always `≤ 1`, because the LP's own capacity
constraint is defined relative to that same `capacity` — a solo consumer can never need more than
what the constraint already bounds it to. So contention always reduces to a clean "keep the
`owned` best of `N` candidates" selection (every contributor needs exactly one dedicated unit,
never more) — not a general knapsack.

**Fix**: extended `find_production_plan`'s existing grower-stranding exclusion loop
(section 46) with a second check, using a new `build_processor_usage` helper (factored out of the
`income_streams`-building loop, now also reused for the final `coin_items` build instead of
duplicating the logic): for each processor facility whose distinct contributor count exceeds its
owned units, keep the `owned` contributors with the highest `rate_per_second` (the right
tie-break, since every contributor needs exactly one unit regardless of its fraction, so ranking
by economic value directly answers "which recipe is worth the unit") and exclude the rest —
merged into the same `excluded` set and re-solve-until-stable loop already used for stranded
grower chains. This is a greedy per-facility choice, not a full joint re-solve over every
combination — consistent with the precedent already accepted for the grower multi-facility
coordination case (documented there as "not necessarily provably-optimal, good enough"), but the
*result* is always fully self-consistent: excluding the loser triggers a genuine re-solve that
correctly cascades its freed-up upstream capacity to a real next-best use (verified live: when
soy_sauce_tofu lost the single Claw Game Cooker slot to strawberry_cream_puff, Farmland's soybean
plots reallocated to selling tofu directly instead of vanishing or sitting idle).

Since the loop now guarantees no settled candidate set ever has processor contention, the
percentage/time-share branch in `coin_items` became provably unreachable and was deleted (kept a
`debug_assert!` rather than an unchecked `unreachable!()`, and `saturating_sub` instead of a bare
subtraction, as cheap insurance against a floating-point edge case at the boundary). Every
processor row now always shows whole-unit dedication or "no profitable item" — never a percentage.

New regression test (replacing an now-obsolete one from section 44 whose whole premise —
"idle Joy Wheel Loom capacity should feed a second item" — was exactly the bug just fixed):
`test_find_coin_plan_processor_contention_dedicates_to_one_recipe_not_both` asserts Joy Wheel Loom
(1 unit, both palm_fabric and cotton_fabric wanting it) produces exactly one of the two, never
describes a time-share percentage, and that the loser's raw-material facility still shows a
`Producing` row rather than going idle. All 26 tests pass, `wasm-pack build` clean, verified live
against the standard 20 Farmland / 10 Woodland / 2 Carousel Mill / 1 Claw Game Cooker scenario.

### 49. Polish pass: rewrote outdated Math/Help modals, removed em-dashes from user-facing copy,
added a Facilities reference page

Requested review of "the other components of the app" after the processor-dedication fix
(section 48) landed. Three findings:

**Math modal was describing dead code.** The "How It Works" modal still documented the old
`calculate_optimal_facility_allocation` binary-search algorithm (divisor-counting trick, naive vs.
optimal split example) from before the section 44 LP rewrite. That function still exists and is
still used by the legacy single-item `optimize()` path, but the main UI (`find_plan`/`time_to_reach`)
hasn't called it since section 23. Rewrote the modal to describe what the app actually computes
now: per-item profit/utilization, the joint `solve_facility_allocation` LP, the grower
largest-remainder apportionment (section 46), processor whole-unit dedication (section 48), and
the `time_to_reach_goal` binary search (section 27).

**Help modal's step 1 was stale.** Still said "Set Your Coin Target" as the first step, left over
from before the plan/goal decoupling (section 26) made the target optional and second. Rewrote the
5-step flow (currency → facilities → modules → calculate → optional goal) and the "Understanding
Results" list to match the current three-part results layout (Your Rate / What Each Facility
Should Do / Total Time).

**Em-dashes in user-visible copy** (hint text under Configuration/Results, and the four "New
facility" tooltips for Grass Blossom Mat/Starfall Hammock/Tidewhisper Sandcastle/Dewy House)
replaced with plain punctuation (periods, colons, semicolons). Left alone: code comments (not
user-facing), and the `—`/`&mdash;` placeholder glyphs used in table cells for "nothing here"
(`renderFacilityPlan`'s empty item_name, the byproduct row's Facility/Profit-sec columns) since
that's a standard table convention, not prose.

**New Facilities reference page** (`web/facilities.html` + `web/facilities.js`): a static,
unfiltered production table per facility (grouped by the same Materials / Aniimo Materials /
Materials Processing categories as the optimizer page), listing every recipe's required level,
inputs, yield, time, sell value, and module requirement — independent of what the user owns.
Backed by a new `get_all_items()` wasm export in `wasm.rs` that dumps every `ProductionItem`'s full
recipe fields as JSON (unlike `get_available_items`, not filtered by owned facility level).

While building this, noticed `app.js` and the new `facilities.js` both needed the same
`FACILITIES`/`FACILITY_CATEGORIES`/`FACILITY_CATEGORY_BY_NAME` config. Rather than duplicate it
(the kind of drift the codebase has explicitly avoided elsewhere — see section 42's "add a
facility in one place" comment), factored it out into a shared `web/facility-config.js` ES module
imported by both. `index.html` gained a "facilities" nav link in the header; `facilities.html`
mirrors its header/theme-toggle/footer structure but has no config form, just the recipe tables.

All 26 Rust tests still pass (no optimizer logic touched), `wasm-pack build` clean, verified live:
Math/Help modal text confirmed via DOM inspection (MathJax renders correctly), a full
facility-count → Calculate → goal run still works end-to-end after the `app.js` refactor, and the
Facilities page renders correct grouped tables with no console errors.

## Facilities status

Full 20-facility list (section 11), grouped by category. Old repo only had the 9 marked
"(old)" in the Notes column — everything else is new territory for this project.

**Materials (raw gathering):**

| Facility | Access confirmed by user? | Notes |
|---|---|---|
| Farmland | Yes | (old) Essentially complete (all levels, seed costs, energy, Quick variants) |
| Woodland | Yes | (old) Essentially complete (all levels, seed costs, energy for 5/12 items, Wood Blocks byproduct) |
| Mineral Pile | Yes | (old) Workload-based, no seed cost, full item table gathered (Shell/Quick Shell/Clay/Quartz/Quick Quartz/Gem) |
| Nimbus Bed | Yes | (old) Workload-based (Petals confirmed); old CSV data otherwise unconfirmed for new beta |
| Grass Blossom Mat | Yes | Implemented (Scales/Quick Scales). Facility level guessed as 1 (unconfirmed). |
| Starfall Hammock | Yes | Implemented (Star, sell value 273). Facility level guessed as 1. |
| Tidewhisper Sandcastle | Yes | Implemented (Pearl 441, Love Bubble 441). Facility level guessed as 1. |
| Dewy House | Yes | Implemented (Aromathyst, sell value 357). Facility level guessed as 1. |

**Materials Processing:**

| Facility | Access confirmed by user? | Notes |
|---|---|---|
| Carousel Mill | Yes | Normal Mode fully implemented (4 items) |
| Jukebox Dryer | Yes | Normal Mode fully implemented (10 items across 4 levels) |
| Crafting Table | Yes | Normal Mode fully implemented (21 items across 5 levels) |
| ~~Dance Pad Polisher~~ | Removed | User confirmed doesn't produce coins/coupons/Bud Tickets — out of scope, fully removed (section 20) |
| ~~Aniipod Maker~~ | Removed | User confirmed doesn't produce coins/coupons/Bud Tickets — out of scope, fully removed (section 20) |
| Phonolfactory Table | Yes | Implemented (14 items across 4 levels). |
| Bouncy Brew Keg | Yes | Implemented (10 items across 3 levels). |
| Claw Game Cooker | Yes | Implemented (19 items across 3 levels). |
| Joy Wheel Loom | Yes | Implemented (6 items across 4 levels — see section 42). |

**Auxiliary Facilities (likely not direct item producers — see section 11):**

| Facility | Access confirmed by user? | Notes |
|---|---|---|
| Storage Unit | No | Likely inventory capacity, not production. |
| Crackle Power Pole | No | Power distribution, gated by Power Module Lvl 1. |
| Crackle Generator | No | Power generation, levels 2-5 gated by Power Module. New way to produce Energy passively. |
| Heat Furnace | No | Hypothesized to enable "warm" growing environment (see section 8/11). |
| Cooling Unit | No | Hypothesized to enable "cold" growing environment (see section 8/11). |
| Sunlamp | No | Hypothesized to enable a "sunny/light" growing environment (see section 8/11). |

## Open questions (blocking full model rewrite)

1. Second efficiency data point on the *same* item, to confirm workload/efficiency → time formula.
2. What determines Aniimo Efficiency % (species match, level vs. recommended, gear)? Do we model
   this as a simple user-supplied % per facility, or something more structured?
3. ~~Seed bag sourcing~~ — RESOLVED: bought with gold, full price list obtained (see sections 6-7).
4. Full currency set: {coins, Bud Tickets} vs {coins, coupons, Bud Tickets}. (Crown/ticket
   currency now confirmed named "Bud Tickets.")
5. ~~Name/details of the new facility~~ — RESOLVED/EXPANDED: full 20-facility list obtained
   (section 11), revealing 11 new facilities total, most still needing item-level data.
6. Whether old mechanics (parallel facility counts, multi-facility levels) still exist as-is or
   changed too. Modules: confirmed still exist (section 10). Fertilizer: RESOLVED — no longer
   exists in the new beta at all (see section 12).
7. Growing environment mechanic (warm/cold/etc.) — is it a hard per-plot constraint or cosmetic?
   Not yet accessible to user.
8. Wood Blocks / Mineral Sand → coin shop conversion rate (not yet known; low priority since
   these are treated as informational secondary output for now).
9. Energy values still missing for: Willow Wood, Bamboo, Palm Bark, Natural Rubber,
   Pine Tree Hardwood (Woodland); and all Mineral Pile / other facilities not yet covered.

## Working design decision (tentative, pending confirmation)

Extend the data model so each production item can optionally carry:
- `base_time` (time in seconds at 100% efficiency — replaces flat `production_time` for
  workload-driven facilities; flat-time facilities keep using `production_time` directly).
- The web UI would gain an `efficiency%` input (default 100) per relevant facility, and the
  optimizer divides `base_time` by `efficiency_fraction` to get effective time, same way it
  already divides by facility count for parallelism.
- `workload` itself stored for reference/display only, not used in time math, unless later
  data proves otherwise.
