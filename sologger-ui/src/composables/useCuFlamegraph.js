// Layout math for the CU flamegraph: turns one transaction's LogsTable rows (invoke
// order + depth) into positioned icicle cells. Framework-free on purpose — the
// component renders what this returns, and src/tests/cuFlamegraph.test.js covers it.

export const FLAME_ROW_HEIGHT = 30;
export const FLAME_PALETTE_SLOTS = 8;
const CANVAS_WIDTH = 1000;

function programOf(row) {
    return row.programId?.programId ?? String(row.programId ?? '');
}

function errorsOf(row) {
    try {
        return JSON.parse(row.errors ?? '[]');
    } catch {
        return [];
    }
}

/**
 * Palette slot per program, in order of first appearance. Slot 0 is the neutral
 * "Other" fold bucket once the palette's eight identity slots are spent — hues are
 * never cycled or generated.
 */
export function assignProgramSlots(rows) {
    const slots = new Map();
    for (const row of rows) {
        const program = programOf(row);
        if (!slots.has(program)) {
            slots.set(program, slots.size < FLAME_PALETTE_SLOTS ? slots.size + 1 : 0);
        }
    }
    return slots;
}

/**
 * Positioned cells for one transaction: x/width in a 0..1000 canvas, y from CPI depth.
 * A parent's span is wide enough to hold its children even when its own CU went
 * unreported, and every cell keeps a minimum visible/hoverable width.
 */
export function buildFlamegraphCells(rows) {
    if (!rows?.length) return [];
    const slotByProgram = assignProgramSlots(rows);

    // Rebuild the CPI tree from invoke order + depth
    const roots = [];
    const path = [];
    rows.forEach((row, index) => {
        const depth = Math.max(row.depth ?? 1, 1);
        path.length = depth - 1;
        const node = {row, index, children: []};
        const siblings = path.reduce((level, i) => level[i].children, roots);
        siblings.push(node);
        path.push(siblings.length - 1);
    });

    const cuOf = node => node.row.computeUnits ?? 0;
    const widthOf = (node, minUnits) => {
        const childrenSum = node.children.reduce((sum, child) => sum + widthOf(child, minUnits), 0);
        return Math.max(cuOf(node), childrenSum, minUnits);
    };

    const rootTotalRaw = roots.reduce((sum, node) => sum + Math.max(cuOf(node), 1), 0);
    const minUnits = Math.max(rootTotalRaw / 200, 1);
    const total = roots.reduce((sum, node) => sum + widthOf(node, minUnits), 0);

    const cells = [];
    const emit = (node, startUnits, depth) => {
        const units = widthOf(node, minUnits);
        const row = node.row;
        const program = programOf(row);
        const errors = errorsOf(row);
        const failed = errors.length > 0;
        const short = program.length > 9 ? program.slice(0, 8) + '…' : program;
        cells.push({
            key: node.index,
            x: (startUnits / total) * CANVAS_WIDTH,
            y: (depth - 1) * FLAME_ROW_HEIGHT,
            width: Math.max((units / total) * CANVAS_WIDTH, 4),
            slot: slotByProgram.get(program) ?? 0,
            label: `${failed ? '✗ ' : ''}${short}${row.instructionName ? ' · ' + row.instructionName : ''}`,
            program,
            instruction: row.instructionName || '',
            cu: row.computeUnits ?? null,
            share: row.computeUnits ? ((row.computeUnits / total) * 100).toFixed(1) : '0.0',
            depth,
            failed,
            errors,
        });
        let childStart = startUnits;
        for (const child of node.children) {
            emit(child, childStart, depth + 1);
            childStart += widthOf(child, minUnits);
        }
    };

    let start = 0;
    for (const node of roots) {
        emit(node, start, 1);
        start += widthOf(node, minUnits);
    }
    return cells;
}

/** Legend entries: swatch slot + short program label, one per unique program. */
export function flamegraphLegend(rows) {
    return [...assignProgramSlots(rows).entries()].map(([program, slot]) => ({
        program,
        slot,
        short: slot === 0 ? 'Other' : program.length > 9 ? program.slice(0, 8) + '…' : program,
    }));
}
