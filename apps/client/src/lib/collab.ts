import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

/** What the connection bar shows: syncing or reconnecting, live, offline, or stopped by the server. */
export type Connection = 'connecting' | 'live' | 'offline' | 'stopped';

/** Why the server ended the session for good, from its close code. */
export type Stop = 'refused' | 'too-large' | 'failed';

/** One open document: its Y.Doc, its provider, and how to close both. */
export interface Collab {
	ydoc: Y.Doc;
	provider: WebsocketProvider;
	destroy: () => void;
}

/** What a page wants to hear from its open document. `onroomclose` fires on 1001, before the reconnect. */
export interface CollabEvents {
	onconnection: (state: Connection) => void;
	onstop: (stop: Stop, reason: string) => void;
	onroomclose?: () => void;
}

const STOPS: Record<number, Stop> = {
	1008: 'refused',
	1009: 'too-large',
	1011: 'failed'
};

/**
 * The stop a close code means, or null when the provider should reconnect with backoff: 1001 (the room closed)
 * and 1013 (too slow) reconnect; 1008 (refused change or access), 1009 (too large) and 1011 (room failed) stop.
 */
export function stopOf(code: number): Stop | null {
	return STOPS[code] ?? null;
}

/** The y-websocket server URL for this origin. The room name `{id}/ws` completes `/api/documents/{id}/ws`. */
export function socketBase(location: Pick<Location, 'protocol' | 'host'>): string {
	const scheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
	return `${scheme}//${location.host}/api/documents`;
}

/** The connection state from the provider's status and the browser's network flag. */
export function connectionOf(status: string, synced: boolean, online: boolean): Connection {
	if (!online) return 'offline';
	return status === 'connected' && synced ? 'live' : 'connecting';
}

/** Opens document `id` over y-websocket, reports every connection change, and stops on a terminal close. */
export function openCollab(id: string, events: CollabEvents): Collab {
	const ydoc = new Y.Doc();
	const provider = new WebsocketProvider(socketBase(window.location), `${encodeURIComponent(id)}/ws`, ydoc, {
		disableBc: true,
		shouldReconnect: (event) => stopOf(event.code) === null
	});
	let status = 'connecting';
	let stopped = false;
	const report = () =>
		events.onconnection(stopped ? 'stopped' : connectionOf(status, provider.synced, navigator.onLine));
	provider.on('status', (event: { status: string }) => {
		status = event.status;
		report();
	});
	provider.on('sync', report);
	provider.on('connection-close', (event: CloseEvent | null) => {
		if (event?.code === 1001) events.onroomclose?.();
	});
	provider.on('closed', (event: { code: number; reason: string }) => {
		const stop = stopOf(event.code);
		if (!stop) return;
		stopped = true;
		report();
		events.onstop(stop, event.reason);
	});
	window.addEventListener('online', report);
	window.addEventListener('offline', report);
	report();
	return {
		ydoc,
		provider,
		destroy: () => {
			window.removeEventListener('online', report);
			window.removeEventListener('offline', report);
			provider.destroy();
			ydoc.destroy();
		}
	};
}
