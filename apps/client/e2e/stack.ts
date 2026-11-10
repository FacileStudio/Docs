import { SQL, type Subprocess } from 'bun';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

/** The people the seed creates: two members of the shared space and one outsider. */
export type Person = 'alice' | 'bob' | 'carol';

/** What a running stack hands the tests: the app's URL, the shared space, one session per person. */
export type Stack = { base: string; space: string; cookies: Record<Person, string>; stop: () => Promise<void> };

const root = resolve(import.meta.dir, '../../..');
const api = resolve(root, 'apps/api/target/debug/api');
const seeder = resolve(root, 'apps/api/target/debug/examples/e2e_seed');
const clientDir = resolve(root, 'apps/client/build');
const admin = process.env.E2E_ADMIN_URL ?? 'postgres://postgres:postgres@127.0.0.1:5654/postgres?sslmode=disable';
const database = 'docs_e2e';

function freePort(): number {
	const probe = Bun.serve({ port: 0, fetch: () => new Response() });
	const port = probe.port ?? 0;
	probe.stop(true);
	return port;
}

async function freshDatabase(): Promise<string> {
	const sql = new SQL(admin);
	await sql.unsafe(`DROP DATABASE IF EXISTS ${database} WITH (FORCE)`);
	await sql.unsafe(`CREATE DATABASE ${database}`);
	await sql.close();
	const url = new URL(admin);
	url.pathname = `/${database}`;
	return url.toString();
}

async function ready(base: string, server: Subprocess): Promise<void> {
	for (let attempt = 0; attempt < 300; attempt += 1) {
		if (server.exitCode !== null) throw new Error(`the API exited with ${server.exitCode}`);
		const answer = await fetch(`${base}/ready`).catch(() => null);
		if (answer?.ok) return;
		await Bun.sleep(100);
	}
	throw new Error('the API did not become ready in 30 seconds');
}

async function seed(env: Record<string, string>): Promise<{ space: string; cookies: Stack['cookies'] }> {
	const run = Bun.spawn([seeder], { env: { ...process.env, ...env }, stdout: 'pipe', stderr: 'inherit' });
	const out = await new Response(run.stdout).text();
	if ((await run.exited) !== 0) throw new Error('e2e_seed failed');
	return JSON.parse(out);
}

/** Starts the Docs API on a fresh database and a free port, serving the built client, then seeds it. */
export async function startStack(): Promise<Stack> {
	for (const path of [api, seeder, `${clientDir}/index.html`]) {
		if (!existsSync(path)) throw new Error(`${path} is missing: run "mise run e2e", which builds it`);
	}
	const port = freePort();
	const base = `http://127.0.0.1:${port}`;
	const dataDir = mkdtempSync(join(tmpdir(), 'docs-e2e-'));
	const env = {
		DATABASE_URL: await freshDatabase(),
		PORT: String(port),
		CLIENT_DIR: clientDir,
		DATA_DIR: dataDir,
		OIDC_ISSUER: '',
		RUST_LOG: process.env.RUST_LOG ?? 'warn'
	};
	const server = Bun.spawn([api], { env: { ...process.env, ...env }, stdout: 'inherit', stderr: 'inherit' });
	await ready(base, server);
	const seeded = await seed(env);
	const stop = async () => {
		server.kill('SIGTERM');
		await server.exited;
		rmSync(dataDir, { recursive: true, force: true });
	};
	return { base, ...seeded, stop };
}
