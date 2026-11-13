import { blockCommands, labelsFor, type Labels } from '@facile/encre';
import type { Editor } from '@tiptap/core';
import { blockIcon, icons } from './icons';
import type { ContextMenuAction, ContextMenuItem } from './components/ContextMenu.svelte';

/** The encre editor methods the right-click menu calls. */
export interface MenuEditor {
	instance(): Editor | null;
	turnInto(id: string): boolean;
	duplicateBlock(): boolean;
	deleteSelection(): boolean;
	copyBlockLink(base: string): string | null;
}

/** What the menu is opened on. */
export interface MenuContext {
	editor: MenuEditor;
	editable: boolean;
	empty: boolean;
	markdown: string;
	/** The page URL a block link starts with. */
	pageUrl: string;
	assistant: string;
	onask: (markdown: string) => void;
	onpaste: () => void;
	oncopy: (cut: boolean) => void;
	oncopied: () => void;
}

const LABELS: Labels = labelsFor('fr');

type Chain = ReturnType<Editor['chain']>;

function running(editor: MenuEditor, go: (chain: Chain) => Chain): () => void {
	return () => {
		const tiptap = editor.instance();
		if (tiptap) go(tiptap.chain().focus()).run();
	};
}

type Run = (go: (chain: Chain) => Chain) => () => void;

function tableMenu(run: Run): ContextMenuAction {
	return {
		label: LABELS.table,
		icon: icons.table,
		items: [
			{ label: LABELS.addRow, icon: icons.plus, onselect: run((c) => c.addRowAfter()) },
			{ label: LABELS.addColumn, icon: icons.plus, onselect: run((c) => c.addColumnAfter()) },
			{ label: LABELS.deleteRow, icon: icons.remove, tone: 'danger', onselect: run((c) => c.deleteRow()) },
			{ label: LABELS.deleteColumn, icon: icons.remove, tone: 'danger', onselect: run((c) => c.deleteColumn()) },
			{ label: LABELS.deleteTable, icon: icons.remove, tone: 'danger', onselect: run((c) => c.deleteTable()) }
		]
	};
}

function columnsMenu(run: Run): ContextMenuAction {
	return {
		label: LABELS.columns,
		icon: icons.columns,
		items: [
			{ label: LABELS.addSide, icon: icons.plus, onselect: run((c) => c.addSide()) },
			{ label: LABELS.deleteSide, icon: icons.remove, tone: 'danger', onselect: run((c) => c.deleteSide()) }
		]
	};
}

function badgeMenu(run: Run): ContextMenuAction {
	const tones = Object.keys(LABELS.tones) as (keyof Labels['tones'])[];
	return {
		label: LABELS.badge,
		icon: icons.badge,
		items: tones.map((tone) => ({
			label: LABELS.tones[tone],
			icon: icons.badge,
			onselect: run((c) => c.setBadge({ tone }))
		}))
	};
}

function structure(context: MenuContext): ContextMenuAction[] {
	const tiptap = context.editor.instance();
	if (!tiptap || !context.editable) return [];
	const run: Run = (go) => running(context.editor, go);
	const found: ContextMenuAction[] = [];
	if (tiptap.isActive('table')) found.push(tableMenu(run));
	if (tiptap.isActive('columns')) found.push(columnsMenu(run));
	if (tiptap.isActive('badge')) found.push(badgeMenu(run));
	return found;
}

/** The entries of the editor's right-click menu, disabled where the page cannot be edited. */
export function editorMenu(context: MenuContext): ContextMenuItem[] {
	const { editor, editable, empty } = context;
	const turn = blockCommands(LABELS).map((command) => ({
		label: command.label,
		icon: blockIcon(command.id),
		onselect: () => void editor.turnInto(command.id)
	}));
	return [
		{ label: 'Couper', icon: icons.cut, disabled: !editable || empty, onselect: () => context.oncopy(true) },
		{ label: 'Copier', icon: icons.copy, disabled: empty, onselect: () => context.oncopy(false) },
		{ label: 'Coller', icon: icons.paste, disabled: !editable, onselect: context.onpaste },
		{ separator: true },
		{ label: 'Transformer en', icon: icons.refresh, disabled: !editable, items: turn },
		...structure(context),
		{ label: 'Dupliquer', icon: icons.copy, disabled: !editable, onselect: () => void editor.duplicateBlock() },
		{
			label: 'Supprimer',
			icon: icons.remove,
			tone: 'danger',
			disabled: !editable,
			onselect: () => void editor.deleteSelection()
		},
		{
			label: 'Copier le lien du bloc',
			icon: icons.link,
			onselect: () => {
				if (editor.copyBlockLink(context.pageUrl)) context.oncopied();
			}
		},
		{ separator: true },
		{
			label: `Demander à ${context.assistant}`,
			blob: context.assistant,
			disabled: context.markdown.trim() === '',
			onselect: () => context.onask(context.markdown)
		}
	];
}
