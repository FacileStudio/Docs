import { readFileSync } from 'node:fs';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import type { PluginOption } from 'vite';
import { defineConfig } from 'vitest/config';

const api = 'http://127.0.0.1:4000';

const encre = JSON.parse(readFileSync('node_modules/@facile/encre/package.json', 'utf8'));
const shared = [...Object.keys(encre.dependencies), ...Object.keys(encre.peerDependencies), 'lib0'];

async function devMock(): Promise<PluginOption[]> {
	if (process.env.DOCS_MOCK !== '1') return [];
	const { mockApiPlugin } = await import('./src/lib/dev/vite-plugin.ts');
	return [mockApiPlugin()];
}

export default defineConfig(async () => ({
	plugins: [tailwindcss(), sveltekit(), ...(await devMock())],
	optimizeDeps: {
		exclude: ['@facile/muse', '@facile/encre']
	},
	resolve: { dedupe: shared, ...(process.env.VITEST ? { conditions: ['browser'] } : {}) },
	server: {
		proxy: process.env.DOCS_MOCK === '1' ? undefined : { '/api': { target: api, changeOrigin: true, ws: true } }
	},
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'jsdom'
	}
}));
