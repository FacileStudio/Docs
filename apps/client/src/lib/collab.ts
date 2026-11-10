import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

/** What the connection bar shows: syncing or reconnecting, live, or offline. */
export type Connection = 'connecting' | 'live' | 'offline';

/** One open document: its Y.Doc, its provider, and how to close both. */
export interface Collab {
	ydoc: Y.Doc;
	provider: WebsocketProvider;
	destroy: () => void;
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

/** Opens document `id` over y-websocket and reports every connection change. */
export function openCollab(id: string, onConnection: (state: Connection) => void): Collab {
	const ydoc = new Y.Doc();
	const provider = new WebsocketProvider(socketBase(window.location), `${encodeURIComponent(id)}/ws`, ydoc);
	let status = 'connecting';
	const report = () => onConnection(connectionOf(status, provider.synced, navigator.onLine));
	provider.on('status', (event: { status: string }) => {
		status = event.status;
		report();
	});
	provider.on('sync', report);
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
