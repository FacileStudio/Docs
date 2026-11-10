import type { Mermaid } from 'mermaid';

let mermaid: Promise<Mermaid> | null = null;
let drawn = 0;

function load(): Promise<Mermaid> {
	mermaid ??= import('mermaid').then(({ default: loaded }) => {
		loaded.initialize({ startOnLoad: false, securityLevel: 'strict', theme: 'neutral' });
		return loaded;
	});
	return mermaid;
}

/**
 * encre's `renderDiagram` hook: draws a mermaid diagram as SVG, loading mermaid on the first one.
 * Any other language rejects, so encre shows its source.
 */
export async function renderDiagram(language: string, source: string): Promise<string> {
	if (language !== 'mermaid') throw new Error(`${language} diagrams are shown as source`);
	const { svg } = await (await load()).render(`docs-diagram-${++drawn}`, source);
	return svg;
}
