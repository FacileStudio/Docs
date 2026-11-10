import type { IncomingMessage, ServerResponse } from 'node:http';
import type { Plugin } from 'vite';
import { MockApi } from './mock-api.ts';

async function toRequest(req: IncomingMessage): Promise<Request> {
	const chunks: Buffer[] = [];
	for await (const chunk of req) chunks.push(chunk as Buffer);
	const headers = new Headers(Object.entries(req.headers).map(([k, v]): [string, string] => [k, String(v)]));
	const hasBody = !['GET', 'HEAD'].includes(req.method ?? 'GET');
	return new Request(`http://mock/api${req.url ?? ''}`, {
		method: req.method,
		headers,
		body: hasBody ? Buffer.concat(chunks) : undefined
	});
}

async function reply(res: ServerResponse, response: Response): Promise<void> {
	res.statusCode = response.status;
	response.headers.forEach((value, key) => res.setHeader(key, value));
	res.end(Buffer.from(await response.arrayBuffer()));
}

/** Serves the mock API under `/api` in `vite dev`, in place of the proxy. Loaded only when `DOCS_MOCK=1`. */
export function mockApiPlugin(): Plugin {
	const api = new MockApi();
	return {
		name: 'docs-mock-api',
		configureServer(server) {
			server.middlewares.use('/api', async (req, res) => reply(res, await api.handle(await toRequest(req))));
		}
	};
}
