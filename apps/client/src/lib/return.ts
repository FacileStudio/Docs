const RETURN_KEY = 'docs.return';

type Store = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

function local(path: string | null): path is string {
	return path !== null && path.startsWith('/') && !path.startsWith('//') && !path.startsWith('/\\');
}

function storage(): Store | null {
	try {
		return sessionStorage;
	} catch {
		return null;
	}
}

/**
 * Keeps the page a signed-out visitor opened, so sign-in can bring them back: porte's `/auth/oidc` always lands
 * on `OIDC_SUCCESS_URL`. The home page clears it.
 */
export function rememberReturn(url: Pick<URL, 'pathname' | 'search'>, store: Store | null = storage()): void {
	const path = `${url.pathname}${url.search}`;
	try {
		if (path === '/' || !local(path)) store?.removeItem(RETURN_KEY);
		else store?.setItem(RETURN_KEY, path);
	} catch {
		return;
	}
}

/** The page kept by `rememberReturn`, read once and cleared; null when none or when it is not a local path. */
export function takeReturn(store: Store | null = storage()): string | null {
	try {
		const path = store?.getItem(RETURN_KEY) ?? null;
		store?.removeItem(RETURN_KEY);
		return local(path) ? path : null;
	} catch {
		return null;
	}
}
