import { Document } from '@facile/encre';
import { flushSync, mount, tick, unmount } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { renderDiagram } from './diagrams';
import { pickLocale } from './locale';

const mermaid = vi.hoisted(() => ({
	loads: 0,
	initialize: vi.fn(),
	render: vi.fn(async (id: string, source: string) => ({ svg: `<svg id="${id}"><text>${source.length}</text></svg>` }))
}));

vi.mock('mermaid', () => {
	mermaid.loads += 1;
	return { default: { initialize: mermaid.initialize, render: mermaid.render } };
});

const diagram = (language: string) => ({
	type: 'doc',
	content: [
		{ type: 'diagram', attrs: { id: 'd1', language }, content: [{ type: 'text', text: 'flowchart LR\n  A --> B' }] }
	]
});

async function settle() {
	for (let i = 0; i < 5; i++) await tick();
	flushSync();
}

describe('renderDiagram', () => {
	it('loads mermaid strictly on the first mermaid diagram and draws it as SVG in the document', async () => {
		await expect(renderDiagram('d2', 'a -> b')).rejects.toThrow('source');
		expect(mermaid.loads).toBe(0);
		const target = document.createElement('div');
		document.body.append(target);
		const app = mount(Document, { target, props: { doc: diagram('mermaid'), renderDiagram } });
		await vi.waitFor(() => expect(target.querySelector('.encre-diagram svg')).not.toBeNull());
		expect(mermaid.initialize).toHaveBeenCalledWith(expect.objectContaining({ securityLevel: 'strict' }));
		expect(mermaid.loads).toBe(1);
		unmount(app);
	});

	it('leaves d2 and dot diagrams to encre’s source view', async () => {
		const target = document.createElement('div');
		const app = mount(Document, { target, props: { doc: diagram('dot'), renderDiagram } });
		await settle();
		expect(target.querySelector('.encre-diagram svg')).toBeNull();
		expect(target.textContent).toContain('A --> B');
		unmount(app);
	});
});

describe('pickLocale', () => {
	it('takes the first language encre speaks, and French otherwise', () => {
		expect(pickLocale(['de-AT', 'en'])).toBe('de');
		expect(pickLocale(['it', 'es-MX'])).toBe('es');
		expect(pickLocale(['ja'])).toBe('fr');
		expect(pickLocale([])).toBe('fr');
	});
});
