<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { initLayout, layoutDocument, type Document, type Node, type Placed } from '@estragon/layout-wasm';
	import CssNode from '#lib/components/CssNode.svelte';
	import hud from '../../../../schema/fixtures/hud.ui.json';

	let source = $state(JSON.stringify(hud, null, 2));
	let ready = $state(false);
	let stageWidth = $state(640);

	// Parse the editor contents; keep the last good document while typing.
	let lastGood: Document = hud as Document;
	const parsed = $derived.by((): { doc: Document; error: string | null } => {
		try {
			lastGood = JSON.parse(source) as Document;
			return { doc: lastGood, error: null };
		} catch (e) {
			return { doc: lastGood, error: (e as Error).message };
		}
	});
	const doc = $derived(parsed.doc);
	const parseError = $derived(parsed.error);

	// Taffy via WASM: the exact numbers Godot's EstragonGrid produces.
	const layout = $derived.by((): { placed: Placed[]; error: string | null } => {
		if (!ready) return { placed: [], error: null };
		try {
			return { placed: layoutDocument(doc), error: null };
		} catch (e) {
			return { placed: [], error: String(e) };
		}
	});
	const placed = $derived(layout.placed);
	const layoutError = $derived(layout.error);

	const fills = $derived.by(() => {
		const map = new Map<string, string | undefined>();
		const walk = (n: Node) => {
			map.set(n.id, n.fill);
			n.children?.forEach(walk);
		};
		walk(doc.root);
		return map;
	});

	const scale = $derived(Math.min(1, (stageWidth - 2) / doc.width)); // 2px = frame border

	// Diff the browser's native CSS Grid render against Taffy.
	let cssStage: HTMLDivElement | undefined = $state();
	let diffs = $state<{ id: string; taffy: Placed; css: DOMRect | null }[]>([]);
	$effect(() => {
		void placed;
		void scale;
		tick().then(() => {
			if (!cssStage) return;
			const origin = cssStage.getBoundingClientRect();
			const r = (v: number) => Math.round(v * 10) / 10;
			diffs = placed
				.map((p) => {
					const el = cssStage!.querySelector<HTMLElement>(`[data-id="${CSS.escape(p.id)}"]`);
					if (!el) return { id: p.id, taffy: p, css: null };
					const b = el.getBoundingClientRect();
					const css = new DOMRect(
						r((b.x - origin.x) / scale),
						r((b.y - origin.y) / scale),
						r(b.width / scale),
						r(b.height / scale)
					);
					return { id: p.id, taffy: p, css };
				})
				.filter(({ taffy: t, css: c }) => {
					if (!c) return true;
					return [t.x - c.x, t.y - c.y, t.width - c.width, t.height - c.height].some((d) => Math.abs(d) > 0.5);
				});
		});
	});

	onMount(async () => {
		await initLayout();
		ready = true;
	});
</script>

<svelte:head><title>Estragon Studio</title></svelte:head>

<main>
	<header>
		<h1>Estragon <span>studio · layout spike</span></h1>
		<p>Stop waiting for Godot UI. Design it on the web.</p>
	</header>

	<section class="editor">
		<label for="doc">Document <code>.ui.json</code></label>
		<textarea id="doc" bind:value={source} spellcheck="false"></textarea>
		{#if parseError}<p class="error">JSON: {parseError}</p>{/if}
		{#if layoutError}<p class="error">Layout: {layoutError}</p>{/if}
	</section>

	<section class="panes" bind:clientWidth={stageWidth}>
		<figure>
			<figcaption>Taffy (WASM) <em>= what Godot renders</em></figcaption>
			<div class="frame" style:width="{doc.width * scale}px" style:height="{doc.height * scale}px">
				<div class="stage" style:width="{doc.width}px" style:height="{doc.height}px" style:scale={scale}>
					{#each placed as p (p.id)}
						<div
							class="box"
							style:left="{p.x}px"
							style:top="{p.y}px"
							style:width="{p.width}px"
							style:height="{p.height}px"
							style:background={fills.get(p.id)}
						>
							<span>{p.id}</span>
						</div>
					{/each}
				</div>
			</div>
		</figure>

		<figure>
			<figcaption>
				Browser CSS Grid
				{#if ready}
					<strong class:ok={diffs.length === 0}>
						{diffs.length === 0 ? `✓ matches Taffy (${placed.length} nodes)` : `✗ ${diffs.length} node(s) differ`}
					</strong>
				{/if}
			</figcaption>
			<div class="frame" style:width="{doc.width * scale}px" style:height="{doc.height * scale}px">
				<div class="stage" bind:this={cssStage} style:width="{doc.width}px" style:height="{doc.height}px" style:scale={scale}>
					<CssNode node={doc.root} isRoot />
				</div>
			</div>
			{#if diffs.length}
				<table>
					<thead><tr><th>node</th><th>taffy x,y,w,h</th><th>css x,y,w,h</th></tr></thead>
					<tbody>
						{#each diffs as d (d.id)}
							<tr>
								<td>{d.id}</td>
								<td>{d.taffy.x}, {d.taffy.y}, {d.taffy.width}, {d.taffy.height}</td>
								<td>{d.css ? `${d.css.x}, ${d.css.y}, ${d.css.width}, ${d.css.height}` : 'missing'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</figure>
	</section>
</main>

<style>
	:global(body) {
		margin: 0;
		background: #0b0e14;
		color: #d6dae3;
		font: 14px/1.4 system-ui, sans-serif;
	}
	main {
		display: grid;
		grid-template-columns: minmax(280px, 380px) minmax(0, 1fr);
		grid-template-rows: auto 1fr;
		grid-template-areas: 'header header' 'editor panes';
		gap: 16px;
		padding: 16px;
		min-height: 100vh;
		box-sizing: border-box;
	}
	header {
		grid-area: header;
	}
	h1 {
		margin: 0;
		font-size: 20px;
	}
	h1 span,
	header p,
	figcaption em {
		color: #7d8596;
		font-weight: 400;
		font-size: 13px;
	}
	header p {
		margin: 2px 0 0;
	}
	.editor {
		grid-area: editor;
		display: grid;
		grid-template-rows: auto 1fr auto;
		gap: 6px;
		min-height: 0;
	}
	textarea {
		font: 12px/1.45 ui-monospace, monospace;
		background: #11151d;
		color: inherit;
		border: 1px solid #252b37;
		border-radius: 6px;
		padding: 10px;
		resize: none;
		min-height: 60vh;
	}
	.error {
		color: #f87171;
		margin: 0;
		font-size: 12px;
	}
	.panes {
		grid-area: panes;
		display: grid;
		gap: 20px;
		align-content: start;
		min-width: 0;
	}
	figure {
		margin: 0;
	}
	figcaption {
		margin-bottom: 6px;
		display: flex;
		gap: 10px;
		align-items: baseline;
	}
	figcaption strong {
		color: #f87171;
		font-size: 12px;
	}
	figcaption strong.ok {
		color: #4ade80;
	}
	.frame {
		box-sizing: content-box;
		overflow: hidden;
		border-radius: 6px;
		border: 1px solid #252b37;
	}
	.stage {
		position: relative;
		transform-origin: 0 0;
		background: #000;
	}
	.box {
		position: absolute;
		box-sizing: border-box;
		outline: 1px solid rgb(255 255 255 / 0.12);
		outline-offset: -1px;
	}
	.box span {
		position: absolute;
		inset: 4px auto auto 6px;
		font: 11px/1 ui-monospace, monospace;
		color: rgb(255 255 255 / 0.75);
	}
	table {
		margin-top: 8px;
		font: 12px ui-monospace, monospace;
		border-collapse: collapse;
	}
	td,
	th {
		padding: 2px 10px 2px 0;
		text-align: left;
	}
	@media (max-width: 900px) {
		main {
			grid-template-columns: minmax(0, 1fr);
			grid-template-areas: 'header' 'panes' 'editor';
		}
	}
</style>
