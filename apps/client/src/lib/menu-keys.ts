/** Whether a `contextmenu` event came from the keyboard (menu key, Shift+F10) rather than a pointer. */
export function fromKeyboard(event: MouseEvent): boolean {
	return typeof PointerEvent !== 'undefined' && event instanceof PointerEvent && event.pointerType === '';
}

/** The entry index a menu key moves to from `index` among `count` entries, or null for any other key. */
export function step(key: string, index: number, count: number): number | null {
	if (key === 'ArrowDown') return index < count - 1 ? index + 1 : 0;
	if (key === 'ArrowUp') return index > 0 ? index - 1 : count - 1;
	if (key === 'Home') return 0;
	if (key === 'End') return count - 1;
	return null;
}
