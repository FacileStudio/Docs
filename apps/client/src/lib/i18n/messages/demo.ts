import type { Row } from '../row';

/** Words shown inside the hero page and the three demo tiles. */
export const demo = {
	'demo.label': [
		'Three people write in the same Docs page at once: Hugo renames a heading, Léa and you each add a line.',
		'Trois personnes écrivent en même temps dans le même document Docs : Hugo renomme un titre, Léa et vous ajoutez chacun une ligne.',
		'Tres personas escriben a la vez en la misma página de Docs: Hugo cambia un título, Léa y tú añadís una línea cada uno.',
		'Drei Personen schreiben gleichzeitig in derselben Docs-Seite: Hugo benennt eine Überschrift um, Léa und Sie fügen je eine Zeile hinzu.'
	],
	'demo.space': ['Team', 'Équipe', 'Equipo', 'Team'],
	'demo.page': ['Autumn launch', 'Lancement d’automne', 'Lanzamiento de otoño', 'Herbststart'],
	'demo.heading': ['Agenda', 'Ordre du jour', 'Orden del día', 'Tagesordnung'],
	'demo.heading.hugo': ['Thursday’s plan', 'Programme de jeudi', 'Plan del jueves', 'Plan für Donnerstag'],
	'demo.line.lea': [
		'Approve the budget with Hugo',
		'Valider le budget avec Hugo',
		'Aprobar el presupuesto con Hugo',
		'Budget mit Hugo abstimmen'
	],
	'demo.line.own': ['Book the room', 'Réserver la salle', 'Reservar la sala', 'Raum buchen'],
	'demo.history': ['History', 'Historique', 'Historial', 'Verlauf'],
	'demo.export': ['Export', 'Exporter', 'Exportar', 'Exportieren'],
	'demo.slash.title': [
		'A block for every idea',
		'Un bloc pour chaque idée',
		'Un bloque para cada idea',
		'Ein Block für jede Idee'
	],
	'demo.slash.body': [
		'Type / to add a heading, a list or a table.',
		'Tapez / pour ajouter un titre, une liste ou un tableau.',
		'Escribe / para añadir un título, una lista o una tabla.',
		'Tippen Sie /, um eine Überschrift, eine Liste oder eine Tabelle einzufügen.'
	],
	'demo.slash.page': ['Winter planning', 'Planning d’hiver', 'Planificación de invierno', 'Winterplanung'],
	'demo.slash.team': ['Team', 'Équipe', 'Equipo', 'Team'],
	'demo.slash.date': ['Date', 'Date', 'Fecha', 'Datum'],
	'demo.slash.design': ['Design', 'Design', 'Diseño', 'Design'],
	'demo.slash.product': ['Product', 'Produit', 'Producto', 'Produkt'],
	'demo.history.title': [
		'Go back to an earlier version',
		'Revenez à une version passée',
		'Vuelve a una versión anterior',
		'Zurück zu einer früheren Version'
	],
	'demo.history.body': [
		'A version for every ten minutes of work. Bring one back in a click.',
		'Une version toutes les dix minutes d’activité. Restaurez-la d’un clic.',
		'Una versión por cada diez minutos de trabajo. Recupérala con un clic.',
		'Eine Version alle zehn Minuten Arbeit. Mit einem Klick zurückholen.'
	],
	'demo.history.today': [
		'Today, 10:40 · latest',
		'Aujourd’hui, 10:40 · la plus récente',
		'Hoy, 10:40 · la más reciente',
		'Heute, 10:40 · neueste'
	],
	'demo.history.yesterday': ['Yesterday, 17:20', 'Hier, 17:20', 'Ayer, 17:20', 'Gestern, 17:20'],
	'demo.history.restore': ['Restore', 'Restaurer', 'Restaurar', 'Wiederherstellen'],
	'demo.history.restored': ['Version restored', 'Version restaurée', 'Versión restaurada', 'Version wiederhergestellt'],
	'demo.embed.title': [
		'A live Feuilles sheet in your page',
		'Un tableau Feuilles vivant dans votre page',
		'Una hoja de Feuilles viva en tu página',
		'Eine Live-Tabelle aus Feuilles in Ihrer Seite'
	],
	'demo.embed.body': [
		'Insert a Feuilles sheet or a Toile board: it stays up to date in the page.',
		'Insérez un tableau Feuilles ou une toile : il reste à jour dans la page.',
		'Inserta una hoja de Feuilles o una pizarra de Toile: se mantiene al día en la página.',
		'Fügen Sie eine Feuilles-Tabelle oder ein Toile-Board ein: Es bleibt in der Seite aktuell.'
	],
	'demo.embed.page': ['Q4 budget review', 'Point budget T4', 'Revisión del presupuesto T4', 'Budgetrunde Q4'],
	'demo.embed.book': ['Q4 budget', 'Budget T4', 'Presupuesto T4', 'Q4-Budget'],
	'demo.embed.hosting': ['Hosting', 'Hébergement', 'Alojamiento', 'Hosting'],
	'demo.embed.design': ['Design', 'Design', 'Diseño', 'Design'],
	'demo.embed.ads': ['Ads', 'Pub', 'Anuncios', 'Werbung'],
	'demo.embed.total': ['Total', 'Total', 'Total', 'Summe']
} satisfies Record<string, Row>;
