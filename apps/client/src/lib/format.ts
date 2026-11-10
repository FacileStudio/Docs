const DATE = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'medium', timeStyle: 'short' });

/** A timestamp from the API as a French date and time, or the raw value when it does not parse. */
export function formatDate(value: string): string {
	const date = new Date(value);
	return Number.isNaN(date.getTime()) ? value : DATE.format(date);
}
