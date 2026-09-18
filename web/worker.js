// Web Worker hosting the wasm optimizer and the HiGHS solver. Solving can take long enough that
// running it on the main thread would freeze the page's own rendering, which is what makes the
// browser offer to kill the tab; every wasm call is dispatched through here instead, so the main
// thread stays free to paint a progress indicator while a solve is in flight. See `web/app.js`'s
// `callWorker` for the request/response contract this expects.
import init, { find_plan, exact_byproduct_problems, exact_problem, exact_plan, time_to_reach, get_version, get_all_items } from './pkg/aniimax.js';
import highsModule from './vendor/highs/highs.mjs';

const ready = init();

// Handlers taking a single string argument and returning one; `find_plan` is handled separately
// below since it also takes a progress callback.
const HANDLERS = { time_to_reach, get_version, get_all_items };

// HiGHS (https://highs.dev), compiled to WebAssembly. A fresh instance per solve, from bytes
// fetched once, so one failed solve can't leave a broken instance behind for the next.
let highsBytes = null;
async function newHighs() {
    if (!highsBytes) {
        highsBytes = fetch(new URL('./vendor/highs/highs.wasm', import.meta.url)).then(r => {
            if (!r.ok) throw new Error(`Could not load HiGHS (${r.status})`);
            return r.arrayBuffer();
        });
    }
    return highsModule({ wasmBinary: await highsBytes });
}

// Seconds HiGHS may search before settling for the best plan found so far.
const EXACT_TIME_LIMIT = 30;

// Solves one exact-planner model with HiGHS: `{ values, proven, objective }`, or null if HiGHS
// found no plan at all.
async function solveModel(problem) {
    const highs = await newHighs();
    const result = highs.solve(problem.lp, { mip_rel_gap: 0, time_limit: EXACT_TIME_LIMIT });
    const proven = result.Status === 'Optimal';
    if (!proven && result.Status !== 'Time limit reached') return null;
    const values = Array.from({ length: problem.variables }, (_, i) => result.Columns['x' + i]?.Primal ?? 0);
    return { values, proven, objective: result.ObjectiveValue };
}

// The exact planner (see `exact_problem` in wasm.rs): builds the model in wasm, solves it with
// HiGHS, and turns the answer back into a plan. With "prioritize byproducts" on, it first finds
// the most of each byproduct the facilities can make and requires the plan to keep that much.
// Returns the plan's JSON, or null if this input isn't covered or the solve didn't produce a
// plan, so the caller can fall back to `find_plan`.
async function exactPlanJson(payload) {
    const floors = [];
    let allProven = true;
    for (const problem of JSON.parse(exact_byproduct_problems(payload))) {
        const most = await solveModel(problem);
        if (!most) return null;
        allProven &&= most.proven;
        floors.push([problem.resource, most.objective]);
    }
    const floorsJson = JSON.stringify(floors);
    const problem = JSON.parse(exact_problem(payload, floorsJson));
    if (!problem.lp) return null;
    const solved = await solveModel(problem);
    if (!solved) return null;
    const proven = solved.proven && allProven;
    let bound = solved.objective;
    if (!proven) {
        // The same model without whole units: the most any plan could earn.
        const relaxed = (await newHighs()).solve(problem.lp.replace(/\nGeneral\n[\s\S]*\nEnd/, '\nEnd'), {});
        bound = relaxed.ObjectiveValue;
    }
    const json = exact_plan(payload, floorsJson, JSON.stringify({ values: solved.values, proven, bound }));
    return JSON.parse(json).success ? json : null;
}

self.onmessage = async (event) => {
    const { id, type, payload } = event.data;
    try {
        await ready;
        if (type === 'find_plan') {
            let result = null;
            try {
                result = await exactPlanJson(payload);
            } catch (error) {
                console.warn('Exact planner failed; using the heuristic planner instead:', error);
            }
            if (!result) {
                // Forwarded straight from the wasm solver's own real trial-solve count (see
                // `find_plan`'s doc comment in wasm.rs); a `type: 'progress'` message, distinct
                // from the final `{ ok, result }` response below, so `app.js`'s `callWorker` can
                // relay it to a live progress bar without resolving the request early.
                const onProgress = (count) => self.postMessage({ id, type: 'progress', count });
                result = find_plan(payload, onProgress);
            }
            self.postMessage({ id, ok: true, result });
            return;
        }
        const handler = HANDLERS[type];
        if (!handler) {
            throw new Error(`Unknown worker request type: ${type}`);
        }
        const result = payload === undefined ? handler() : handler(payload);
        self.postMessage({ id, ok: true, result });
    } catch (error) {
        self.postMessage({ id, ok: false, error: error && error.message ? error.message : String(error) });
    }
};
