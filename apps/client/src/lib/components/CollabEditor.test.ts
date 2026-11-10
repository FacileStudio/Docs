import { flushSync, mount, tick, unmount } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Awareness } from 'y-protocols/awareness';
import * as Y from 'yjs';
import CollabEditor from './CollabEditor.svelte';

const opened = vi.hoisted(() => [] as { url: string; room: string; doc: Y.Doc; destroyed: boolean }[]);

vi.mock('y-websocket', () => ({
	WebsocketProvider: class {
		awareness: Awareness;
		synced = true;
		private record: (typeof opened)[number];
		constructor(url: string, room: string, doc: Y.Doc) {
			this.awareness = new Awareness(doc);
			this.record = { url, room, doc, destroyed: false };
			opened.push(this.record);
		}
		on() {}
		off() {}
		destroy() {
			this.record.destroyed = true;
			this.awareness.destroy();
		}
	}
}));

let app: ReturnType<typeof mount> | null = null;

afterEach(() => {
	if (app) unmount(app);
	app = null;
	opened.length = 0;
	document.body.innerHTML = '';
});

async function open(editable: boolean) {
	const target = document.createElement('div');
	document.body.append(target);
	const states: string[] = [];
	app = mount(CollabEditor, {
		target,
		props: {
			documentId: 'doc-1',
			editable,
			user: { name: 'Camille' },
			events: {
				onconnection: (state: string) => states.push(state),
				onstop: () => undefined
			}
		}
	});
	flushSync();
	await tick();
	return { target, states };
}

describe('CollabEditor', () => {
	it('mounts the encre editor on the document room with collaboration', async () => {
		const { target } = await open(true);
		expect(opened).toHaveLength(1);
		expect(opened[0].url).toBe(`ws://${window.location.host}/api/documents`);
		expect(opened[0].room).toBe('doc-1/ws');
		const prose = target.querySelector('.ProseMirror');
		expect(prose).not.toBeNull();
		expect(prose?.getAttribute('contenteditable')).toBe('true');
		expect(target.querySelector('[aria-label="Editing now"]')).not.toBeNull();
		const paragraph = new Y.XmlElement('paragraph');
		paragraph.insert(0, [new Y.XmlText('Bonjour depuis le serveur')]);
		opened[0].doc.getXmlFragment('default').insert(0, [paragraph]);
		await tick();
		expect(prose?.textContent).toContain('Bonjour depuis le serveur');
	});

	it('gives a viewer a read-only editor', async () => {
		const { target } = await open(false);
		expect(target.querySelector('.ProseMirror')?.getAttribute('contenteditable')).toBe('false');
	});

	it('closes the room when it unmounts', async () => {
		await open(true);
		unmount(app!);
		app = null;
		expect(opened[0].destroyed).toBe(true);
	});
});
