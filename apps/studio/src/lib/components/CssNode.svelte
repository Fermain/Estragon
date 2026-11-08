<script lang="ts">
	import type { Node } from '@estragon/layout-wasm';
	import { toCss } from '../css';
	import CssNode from './CssNode.svelte';

	let { node, isRoot = false }: { node: Node; isRoot?: boolean } = $props();
</script>

<div class="node" data-id={node.id} style={toCss(node, isRoot)}>
	{#if !node.children?.length}<span class="label">{node.id}</span>{/if}
	{#each node.children ?? [] as child (child.id)}
		<CssNode node={child} />
	{/each}
</div>

<style>
	.node {
		position: relative;
		outline: 1px solid rgb(255 255 255 / 0.12);
		outline-offset: -1px;
	}
	.label {
		position: absolute;
		inset: 4px auto auto 6px;
		font: 11px/1 ui-monospace, monospace;
		color: rgb(255 255 255 / 0.75);
		pointer-events: none;
	}
</style>
