import { describe, expect, it } from 'vitest';
import { connectionOf, socketBase } from './collab';
import { imageFiles } from './images';
import { titleOf } from './import';
import { accessFor } from './session.svelte';
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

	it('grants access by scope role', () => {
		expect(accessFor('personal')).toBe('manage');
		expect(accessFor('owner')).toBe('manage');
		expect(accessFor('admin')).toBe('manage');
		expect(accessFor('member')).toBe('edit');
		expect(accessFor(undefined)).toBe('view');
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
