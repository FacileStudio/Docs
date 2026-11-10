import { describe, expect, it } from 'vitest';
import { connectionOf, socketBase, stopOf } from './collab';
import { imageFiles } from './images';
import { titleOf } from './import';
import { scopeAccess } from './session.svelte';
import { snippetParts } from './snippet';

describe('helpers', () => {
	it('builds the y-websocket URL of the origin', () => {
		expect(socketBase({ protocol: 'https:', host: 'docs.facile.studio' })).toBe(
			'wss://docs.facile.studio/api/documents'
		);
		expect(socketBase({ protocol: 'http:', host: 'localhost:5173' })).toBe('ws://localhost:5173/api/documents');
	});

	it('shows live only once connected and synced, offline whenever the browser is', () => {
		expect(connectionOf('connected', true, true)).toBe('live');
		expect(connectionOf('connected', false, true)).toBe('connecting');
		expect(connectionOf('disconnected', false, true)).toBe('connecting');
		expect(connectionOf('connected', true, false)).toBe('offline');
	});

	it('stops on refused, too large and failed rooms, and reconnects on the others', () => {
		expect(stopOf(1008)).toBe('refused');
		expect(stopOf(1009)).toBe('too-large');
		expect(stopOf(1011)).toBe('failed');
		expect(stopOf(1001)).toBeNull();
		expect(stopOf(1013)).toBeNull();
		expect(stopOf(1006)).toBeNull();
	});

	it('grants access by scope role', () => {
		expect(scopeAccess('personal')).toBe('manager');
		expect(scopeAccess('owner')).toBe('manager');
		expect(scopeAccess('admin')).toBe('manager');
		expect(scopeAccess('member')).toBe('editor');
		expect(scopeAccess(undefined)).toBe('viewer');
	});

	it('splits a ts_headline snippet without HTML', () => {
		expect(snippetParts('a <b>hit</b> &lt;script&gt;')).toEqual([
			{ text: 'a ', hit: false },
			{ text: 'hit', hit: true },
			{ text: ' <script>', hit: false }
		]);
	});

	it('keeps only accepted image types', () => {
		const files = [
			new File(['a'], 'a.png', { type: 'image/png' }),
			new File(['b'], 'b.svg', { type: 'image/svg+xml' })
		];
		const data = { files } as unknown as DataTransfer;
		expect(imageFiles(data).map((f) => f.name)).toEqual(['a.png']);
		expect(imageFiles(null)).toEqual([]);
	});

	it('titles an imported file after its name', () => {
		expect(titleOf('Notes de réunion.md')).toBe('Notes de réunion');
		expect(titleOf('.md')).toBe('Sans titre');
	});
});
