import type { Node, NodeStyle } from '@estragon/layout-wasm';

/** Map an Estragon style to real browser CSS, for the "native CSS Grid" comparison pane. */
export function toCss(node: Node, isRoot: boolean): string {
	const s: NodeStyle = node.style ?? {};
	const css: Record<string, string | number | undefined> = {
		'box-sizing': 'border-box',
		background: node.fill,
		'min-width': `${s.minWidth ?? 0}px`,
		'min-height': `${s.minHeight ?? 0}px`,
		'grid-area': s.area,
		'grid-column': s.column,
		'grid-row': s.row,
		'justify-self': s.justifySelf,
		'align-self': s.alignSelf
	};
	if (isRoot) Object.assign(css, { width: '100%', height: '100%' });
	if (node.kind === 'grid') {
		Object.assign(css, {
			display: 'grid',
			'grid-template-columns': s.columns,
			'grid-template-rows': s.rows,
			'grid-template-areas': s.areas?.map((r) => JSON.stringify(r)).join(' '),
			'grid-auto-rows': s.autoRows,
			'grid-auto-columns': s.autoColumns,
			'row-gap': `${s.rowGap ?? s.gap ?? 0}px`,
			'column-gap': `${s.columnGap ?? s.gap ?? 0}px`,
			padding: `${s.padding ?? 0}px`
		});
	}
	return Object.entries(css)
		.filter(([, v]) => v !== undefined && v !== '')
		.map(([k, v]) => `${k}:${v}`)
		.join(';');
}
