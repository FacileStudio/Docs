/** One run of a search snippet: plain text, or a hit that `ts_headline` wrapped in `<b>`. */
export interface SnippetPart {
	text: string;
	hit: boolean;
}

const ENTITIES: Record<string, string> = { '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"', '&#39;': "'" };

/** Splits a `ts_headline` snippet into text runs without ever handing it to the DOM as HTML. */
export function snippetParts(snippet: string): SnippetPart[] {
	const decode = (text: string) => text.replace(/&(amp|lt|gt|quot|#39);/g, (entity) => ENTITIES[entity]);
	return snippet
		.split(/(<b>.*?<\/b>)/g)
		.filter(Boolean)
		.map((part) => {
			const hit = part.startsWith('<b>') && part.endsWith('</b>');
			return { text: decode(hit ? part.slice(3, -4) : part), hit };
		});
}
