import {describe, expect, it} from 'vitest';
import {
    assignProgramSlots,
    buildFlamegraphCells,
    flamegraphLegend,
    FLAME_ROW_HEIGHT
} from '../composables/useCuFlamegraph';

function row(programId, depth, computeUnits, extras = {}) {
    return {
        programId: {programId},
        depth,
        computeUnits,
        instructionName: '',
        errors: '[]',
        ...extras
    };
}

// CLMM(90232 CU) -> [Tokenkeg(2968), AToken(20293) -> Tokenkeg(1622)], invoke order
function cpiFixture() {
    return [
        row('CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR', 1, 90232, {instructionName: 'OpenPosition'}),
        row('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', 2, 2968, {instructionName: 'InitializeMint'}),
        row('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL', 2, 20293),
        row('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', 3, 1622, {instructionName: 'GetAccountDataSize'}),
    ];
}

describe('buildFlamegraphCells', () => {
    it('returns no cells for empty input', () => {
        expect(buildFlamegraphCells([])).toEqual([]);
        expect(buildFlamegraphCells(null)).toEqual([]);
    });

    it('nests children inside their parent span, one row per depth', () => {
        const cells = buildFlamegraphCells(cpiFixture());
        expect(cells).toHaveLength(4);

        const [clmm, tokenkeg, atoken, nestedTokenkeg] = cells;
        // Root spans the whole canvas on row 1
        expect(clmm.y).toBe(0);
        expect(clmm.x).toBeCloseTo(0);
        expect(clmm.width).toBeCloseTo(1000);

        // Direct children sit on row 2, sequentially, inside the parent span
        expect(tokenkeg.y).toBe(FLAME_ROW_HEIGHT);
        expect(tokenkeg.x).toBeCloseTo(0);
        expect(atoken.y).toBe(FLAME_ROW_HEIGHT);
        expect(atoken.x).toBeGreaterThanOrEqual(tokenkeg.x + tokenkeg.width - 0.01);
        expect(atoken.x + atoken.width).toBeLessThanOrEqual(clmm.x + clmm.width + 0.01);

        // The grandchild nests inside AToken's span on row 3
        expect(nestedTokenkeg.y).toBe(FLAME_ROW_HEIGHT * 2);
        expect(nestedTokenkeg.x).toBeGreaterThanOrEqual(atoken.x - 0.01);
        expect(nestedTokenkeg.x + nestedTokenkeg.width).toBeLessThanOrEqual(atoken.x + atoken.width + 0.01);

        // Widths are CU-proportional
        expect(atoken.width / tokenkeg.width).toBeCloseTo(20293 / 2968, 1);
    });

    it('gives a parent with unreported CU enough span to hold its children', () => {
        const cells = buildFlamegraphCells([
            row('Parent11111111111111111111111111111111111111', 1, null),
            row('ChildA111111111111111111111111111111111111111', 2, 30000),
            row('ChildB111111111111111111111111111111111111111', 2, 10000),
        ]);
        const [parent, childA, childB] = cells;
        expect(parent.width).toBeGreaterThanOrEqual(childA.width + childB.width - 0.01);
        expect(parent.cu).toBeNull();
    });

    it('keeps a minimum visible width for zero-CU invocations', () => {
        const cells = buildFlamegraphCells([
            row('BigProgram1111111111111111111111111111111111', 1, 400000),
            row('TinyProgram111111111111111111111111111111111', 1, null),
        ]);
        expect(cells[1].width).toBeGreaterThanOrEqual(4);
    });

    it('marks failed invocations with the glyph, not a repaint', () => {
        const cells = buildFlamegraphCells([
            row('FailingProgram111111111111111111111111111111', 1, 5000, {
                errors: JSON.stringify(['custom program error: 0x1770']),
                instructionName: 'Trade'
            }),
        ]);
        expect(cells[0].failed).toBe(true);
        expect(cells[0].label.startsWith('✗ ')).toBe(true);
        expect(cells[0].errors).toEqual(['custom program error: 0x1770']);
        // Identity slot is still assigned normally
        expect(cells[0].slot).toBe(1);
    });
});

describe('assignProgramSlots', () => {
    it('assigns hues by first appearance and never cycles', () => {
        const rows = [];
        for (let i = 0; i < 10; i++) {
            rows.push(row(`Program${i}1111111111111111111111111111111111`, 1, 100));
        }
        const slots = assignProgramSlots(rows);
        // First eight get identity slots 1..8 in order
        for (let i = 0; i < 8; i++) {
            expect(slots.get(`Program${i}1111111111111111111111111111111111`)).toBe(i + 1);
        }
        // Everything past the palette folds into "Other" (slot 0), never a new hue
        expect(slots.get('Program81111111111111111111111111111111111')).toBe(0);
        expect(slots.get('Program91111111111111111111111111111111111')).toBe(0);
    });

    it('keeps one slot per program across repeated invocations', () => {
        const slots = assignProgramSlots(cpiFixture());
        expect(slots.size).toBe(3);
        expect(slots.get('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')).toBe(2);
    });
});

describe('flamegraphLegend', () => {
    it('lists each program once with its slot and short label', () => {
        const legend = flamegraphLegend(cpiFixture());
        expect(legend).toHaveLength(3);
        expect(legend[0]).toEqual({
            program: 'CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR',
            slot: 1,
            short: 'CLMM9tUo…'
        });
    });
});
