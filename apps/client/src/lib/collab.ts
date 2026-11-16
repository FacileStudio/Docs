import { connectionOf, openRoom, socketUrl, stopOf, type Connection, type Stop } from '@facile/salon';
import type { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

export { connectionOf, stopOf, type Connection, type Stop };

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

/** The y-websocket server URL for this origin. The room name `{id}/ws` completes `/api/documents/{id}/ws`. */
export function socketBase(location: Pick<Location, 'protocol' | 'host'>): string {
	return socketUrl('/api/documents', location);
}

/** Opens document `id` over salon, reports every connection change, and stops on a terminal close. */
export function openCollab(id: string, events: CollabEvents): Collab {
	const ydoc = new Y.Doc();
	const room = openRoom({ base: socketBase(window.location), room: `${encodeURIComponent(id)}/ws` }, ydoc, events);
	return {
		ydoc,
		provider: room.provider,
		destroy: () => {
			room.destroy();
			ydoc.destroy();
		}
	};
}
