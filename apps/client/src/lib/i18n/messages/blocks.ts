import type { Row } from '../row';

/** Copy for the suite connections, the plan note and the developer block. */
export const blocks = {
	'landing.with.line': [
		'Your sheets, boards and stats live inside your pages.',
		'Vos tableaux, vos toiles et vos statistiques vivent dans vos pages.',
		'Tus hojas, pizarras y estadísticas viven dentro de tus páginas.',
		'Ihre Tabellen, Boards und Statistiken leben in Ihren Seiten.'
	],
	'landing.with.feuilles': [
		'A live sheet in the middle of your page.',
		'Un tableau vivant au milieu de votre page.',
		'Una hoja viva en medio de tu página.',
		'Eine lebendige Tabelle mitten in der Seite.'
	],
	'landing.with.toile': [
		'Your whiteboards, shown in the page.',
		'Vos tableaux blancs, affichés dans la page.',
		'Tus pizarras, mostradas en la página.',
		'Ihre Whiteboards direkt in der Seite.'
	],
	'landing.with.vision': [
		'Your site stats inside a document.',
		'Les statistiques de votre site dans un document.',
		'Las estadísticas de tu web en un documento.',
		'Die Statistiken Ihrer Website im Dokument.'
	],
	'landing.with.bulle': [
		'An assistant next to your text.',
		'Un assistant à côté de votre texte.',
		'Un asistente junto a tu texto.',
		'Ein Assistent neben Ihrem Text.'
	],
	'landing.plan.included': [
		'Docs is included, the free plan too.',
		'Docs est inclus, offre gratuite comprise.',
		'Docs está incluido, también en el plan gratuito.',
		'Docs ist enthalten, auch im kostenlosen Tarif.'
	],
	'landing.plan.paid': [
		'Docs is included, with Bulle next to your pages.',
		'Docs est inclus, avec Bulle à côté de vos pages.',
		'Docs está incluido, con Bulle junto a tus páginas.',
		'Docs ist enthalten, mit Bulle neben Ihren Seiten.'
	],
	'landing.dev.line': [
		'Bulle and your agents read your pages over MCP. Open source, and you can run it yourself.',
		'Bulle et vos agents lisent vos pages via MCP. Le code est ouvert, vous pouvez l’installer chez vous.',
		'Bulle y tus agentes leen tus páginas por MCP. Código abierto, y puedes instalarlo tú mismo.',
		'Bulle und Ihre Agenten lesen Ihre Seiten über MCP. Open Source, auch selbst betreibbar.'
	],
	'landing.dev.mcp': [
		'Point an MCP client at docs.facile.studio/mcp and sign in with Facile.',
		'Branchez un client MCP sur docs.facile.studio/mcp et connectez-vous avec Facile.',
		'Conecta un cliente MCP a docs.facile.studio/mcp e inicia sesión con Facile.',
		'Verbinden Sie einen MCP-Client mit docs.facile.studio/mcp und melden Sie sich mit Facile an.'
	],
	'landing.dev.api': [
		'The REST API reference, route by route.',
		'La référence de l’API REST, route par route.',
		'La referencia de la API REST, ruta por ruta.',
		'Die REST-API-Referenz, Route für Route.'
	],
	'landing.dev.source': [
		'The code is on GitHub under AGPL-3.0.',
		'Le code est sur GitHub, sous licence AGPL-3.0.',
		'El código está en GitHub, con licencia AGPL-3.0.',
		'Der Code liegt auf GitHub, unter AGPL-3.0.'
	]
} satisfies Record<string, Row>;
