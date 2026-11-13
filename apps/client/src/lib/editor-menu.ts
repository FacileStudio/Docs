import { blockCommands, labelsFor, type Labels } from '@facile/encre';
import type { Editor } from '@tiptap/core';
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

function structure(context: MenuContext): ContextMenuAction[] {
	const tiptap = context.editor.instance();
	if (!tiptap || !context.editable) return [];
	const run = (go: (chain: Chain) => Chain) => running(context.editor, go);
	const found: ContextMenuAction[] = [];
	if (tiptap.isActive('table')) {
		found.push({
			label: LABELS.table,
			items: [
				{ label: LABELS.addRow, onselect: run((c) => c.addRowAfter()) },
				{ label: LABELS.addColumn, onselect: run((c) => c.addColumnAfter()) },
				{ label: LABELS.deleteRow, tone: 'danger', onselect: run((c) => c.deleteRow()) },
				{ label: LABELS.deleteColumn, tone: 'danger', onselect: run((c) => c.deleteColumn()) },
				{ label: LABELS.deleteTable, tone: 'danger', onselect: run((c) => c.deleteTable()) }
			]
		});
	}
	if (tiptap.isActive('columns')) {
		found.push({
			label: LABELS.columns,
			items: [
				{ label: LABELS.addSide, onselect: run((c) => c.addSide()) },
				{ label: LABELS.deleteSide, tone: 'danger', onselect: run((c) => c.deleteSide()) }
			]
		});
	}
	if (tiptap.isActive('badge')) {
		const tones = Object.keys(LABELS.tones) as (keyof Labels['tones'])[];
		found.push({
			label: LABELS.badge,
			items: tones.map((tone) => ({ label: LABELS.tones[tone], onselect: run((c) => c.setBadge({ tone })) }))
		});
	}
	return found;
}

/** The entries of the editor's right-click menu, disabled where the page cannot be edited. */
export function editorMenu(context: MenuContext): ContextMenuItem[] {
	const { editor, editable, empty } = context;
	const turn = blockCommands(LABELS).map((command) => ({
		label: command.label,
		onselect: () => void editor.turnInto(command.id)
	}));
	return [
		{ label: 'Couper', disabled: !editable || empty, onselect: () => context.oncopy(true) },
		{ label: 'Copier', disabled: empty, onselect: () => context.oncopy(false) },
		{ label: 'Coller', disabled: !editable, onselect: context.onpaste },
		{ separator: true },
		{ label: 'Transformer en', disabled: !editable, items: turn },
		...structure(context),
		{ label: 'Dupliquer', disabled: !editable, onselect: () => void editor.duplicateBlock() },
		{ label: 'Supprimer', tone: 'danger', disabled: !editable, onselect: () => void editor.deleteSelection() },
		{
			label: 'Copier le lien du bloc',
			onselect: () => {
				if (editor.copyBlockLink(context.pageUrl)) context.oncopied();
			}
		},
		{ separator: true },
		{
			label: `Demander à ${context.assistant}`,
			disabled: context.markdown.trim() === '',
			onselect: () => context.onask(context.markdown)
		}
	];
}
