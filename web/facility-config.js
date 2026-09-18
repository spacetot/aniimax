// Shared facility configuration, used by app.js for both the facility input cards and the
// facility recipe reference modal, so the two stay in sync automatically.

// Facility configuration. `name` must exactly match the facility string used throughout the
// Rust data model (ProductionItem.facility / FacilityCounts keys) since it's sent verbatim as
// the JSON key for each facility's count/level. Add new facilities here only; cards and input
// handling are generated dynamically, no other file needs to change. `category` groups the cards
// in the UI (see `FACILITY_CATEGORIES` below for display order). `hasLevels: false` hides the
// Level input entirely for facilities that don't level up in-game; omit the field (defaults to
// leveled) for any facility that does. `hasWorker: true` adds the Aniimo level and personality
// bonus inputs for facilities an Aniimo works; they set how fast its workload is completed.
// `ability` is the Aniimo ability the facility uses, shown on the level input; `personality` is
// the personality that gets its +20% bonus, shown on the bonus checkbox (omitted if not known).
//
// Only facilities whose data has been verified against the full release are listed; the rest
// return as their data is confirmed.
export const FACILITIES = [
    {
        name: 'Farmland', slug: 'farmland', defaultCount: 1, category: 'Materials',
        tooltip: "Lv.1: wheat&#10;Lv.2: potato, quick wheat&#10;Lv.3: rice, soybean&#10;Lv.4: rose, cotton, quick potato&#10;Lv.5: strawberry, lavender, sugarcane&#10;Lv.6: ginseng, grape, premium wheat, quick rice&#10;Lv.7: cranberry, agave, quick strawberry"
    },
    {
        name: 'Woodland', slug: 'woodland', defaultCount: 1, category: 'Materials',
        tooltip: "Lv.1: willow wood&#10;Lv.2: bamboo, lemon&#10;Lv.3: cherry blossom, apple, maple syrup, quick bamboo&#10;Lv.4: palm bark, chestnut, walnut, quick lemon&#10;Lv.5: natural rubber, coconut, quick maple syrup&#10;Lv.6: cocoa, orange flower, quick coconut&#10;Also yields Wood Blocks"
    },
    {
        name: 'Mine', slug: 'mine', defaultCount: 1, category: 'Materials', hasWorker: true, ability: 'Earth', personality: 'Playful',
        tooltip: "Lv.1: rock&#10;Lv.2: clay&#10;Lv.3: shell&#10;Lv.4: copper ore&#10;Lv.5: quartz ore&#10;Lv.6: gem&#10;Also yields Mineral Sand."
    },
    {
        name: 'Well', slug: 'well', defaultCount: 0, category: 'Materials', hasWorker: true, ability: 'Water', personality: 'Faithful',
        tooltip: "Lv.1: well water, quick well water&#10;Lv.2: fresh water&#10;Lv.3: quick fresh water&#10;Lv.4: deep rock spring water, quick deep rock spring water&#10;Lv.5: natural mineral spring water&#10;Lv.6: quick natural mineral spring water"
    },
    {
        name: 'Tidewhisper Sandcastle', slug: 'tidewhisper-sandcastle', defaultCount: 0, category: 'Aniimo Materials', hasWorker: true, ability: 'Leisure', personality: 'Judicious',
        tooltip: "Lv.1: sea salt&#10;Lv.2: quick sea salt&#10;Lv.3: pearl (needs Warm)"
    },
    {
        name: 'Heat Furnace', slug: 'heat-furnace', defaultCount: 0, category: 'Environment', hasLevels: false,
        tooltip: "Provides Warm or Scorching growing conditions for crops that need one&#10;The calculator picks whichever mode is more profitable&#10;Covers a 9x9 area around itself; how many plots fit depends on what shares it"
    },
    {
        name: 'Cooling Unit', slug: 'cooling-unit', defaultCount: 0, category: 'Environment', hasLevels: false,
        tooltip: "Provides Cool or Freeze growing conditions for crops that need one&#10;The calculator picks whichever mode is more profitable&#10;Covers a 9x9 area around itself; how many plots fit depends on what shares it"
    },
    {
        name: 'Sunlamp', slug: 'sunlamp', defaultCount: 0, category: 'Environment', hasLevels: false,
        tooltip: "Provides Adequate growing conditions for crops that need one&#10;Covers a 9x9 area around itself; how many plots fit depends on what shares it"
    },
    {
        name: 'Carousel Mill', slug: 'carousel-mill', defaultCount: 1, category: 'Materials Processing', hasWorker: true, ability: 'Wind', personality: 'Tenacious',
        tooltip: "Lv.1: wheatmeal&#10;Lv.2: tofu, milled rice&#10;Lv.3: lavender powder&#10;Lv.4: rice drink, ginseng powder&#10;Lv.5: refined flour, coconut oil&#10;Lv.6: cocoa powder, coconut milk"
    },
    {
        name: 'Crafting Table', slug: 'crafting-table', defaultCount: 1, category: 'Materials Processing', hasWorker: true, ability: 'Artisanship', personality: 'Judicious',
        tooltip: "Lv.1: wood sculpture&#10;Lv.2: bamboo ware, river-washed stones, premium river-washed stones&#10;Lv.3: rose freshener, pottery, premium rose freshener&#10;Lv.4: bouquet, shell ornament, lavender sachet&#10;Lv.5: wind chime, star wish lantern, dream catcher, advanced wind chime&#10;Lv.6: rubber duck, pearl necklace, woven toy, porcelain&#10;Lv.7: dye, gemstone dust, flowers in a bottle, advanced gemstone dust&#10;Lv.8: doll&#10;Some recipes need ingredients from facilities not yet in the calculator"
    },
    {
        name: 'Claw Game Cooker', slug: 'claw-game-cooker', defaultCount: 1, category: 'Materials Processing', hasWorker: true, ability: 'Fire', personality: 'Practical',
        tooltip: "Lv.1: bread, premium bread&#10;Lv.2: roasted soybeans&#10;Lv.3: maple candy roasted potatoes, apple tart, rose shortbread&#10;Lv.4: lavender cookies, apple candy&#10;Lv.5: grape candy, caramel nut chips&#10;Lv.6: maple candy star, coconut cookie&#10;Lv.7: flower bread, berry chocolate coconut pudding, premium berry chocolate coconut pudding&#10;Some recipes need ingredients from facilities not yet in the calculator"
    },
    {
        name: 'Jukebox Dryer', slug: 'jukebox-dryer', defaultCount: 1, category: 'Materials Processing', hasWorker: true, ability: 'Dark', personality: 'Nimble',
        tooltip: "Lv.1: potato chips&#10;Lv.2: dried lemon slices&#10;Lv.3: dried cherry blossom, dried bean curd&#10;Lv.4: dried apple slices, dried strawberries&#10;Lv.5: nuts, dried ginseng&#10;Lv.6: dried grapes, shredded coconut&#10;Lv.7: dried cranberries, dried flowers"
    },
    {
        name: 'Simmering Pot', slug: 'simmering-pot', defaultCount: 0, category: 'Materials Processing', hasWorker: true, ability: 'Fire', personality: 'Tenacious',
        tooltip: "Lv.1: plain rice porridge&#10;Lv.2: rose concentrate&#10;Lv.3: rock candy, strawberry jam, maple candy apple jam&#10;Lv.4: chestnut puree, grape jam, ginseng porridge&#10;Lv.5: maple sugar chunk, malt sugar&#10;Lv.6: cocoa spread, cranberry jam, agave syrup"
    },
];

// Display order for facility categories. Auxiliary facilities (Storage Unit, power/climate
// buildings) are deliberately excluded here: they don't produce items.
export const FACILITY_CATEGORIES = ['Materials', 'Environment', 'Aniimo Materials', 'Materials Processing'];

// Facility name -> category, so other pages can group by the same categories as the facility
// input cards (Materials/Aniimo Materials are grower facilities, Materials Processing is processor
// facilities).
export const FACILITY_CATEGORY_BY_NAME = new Map(FACILITIES.map(f => [f.name, f.category]));
