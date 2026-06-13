export type SuggestionItem = {
    label: string;
    kind: "keyword" | "table" | "column";
    detail?: string;
};

export type SchemaData = {
    tables: string[];
    columns: { [tableName: string]: { name: string; type: string }[] };
};

const KEYWORDS = [
    "SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE", "JOIN", "ON",
    "GROUP", "ORDER BY", "HAVING", "LIMIT", "AS", "AND", "OR", "NOT", "IN",
    "LIKE", "IS", "NULL", "INNER", "LEFT", "RIGHT", "OUTER", "CROSS", "SET",
    "VALUES", "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "ORDER", "BY"
];

export function getSuggestions(
    query: string,
    cursorPos: number,
    schema: SchemaData
): SuggestionItem[] {
    const textBeforeCursor = query.slice(0, cursorPos);
    
    // Extract current token at cursorPos (alphanumeric and underscores)
    const currentTokenMatch = textBeforeCursor.match(/([a-zA-Z0-9_]+)$/);
    const currentToken = currentTokenMatch ? currentTokenMatch[1] : "";

    if (!currentToken || currentToken.length < 2) {
        return [];
    }

    const lowerToken = currentToken.toLowerCase();

    // Find previous meaningful token
    const textBeforeCurrentToken = textBeforeCursor.slice(0, -currentToken.length);
    const previousWords = textBeforeCurrentToken.match(/[a-zA-Z0-9_]+/g) || [];
    const previousMeaningfulToken = previousWords.length > 0 ? previousWords[previousWords.length - 1] : "";

    // Check if the previous meaningful token corresponds to a table in the schema
    let matchingTable: string | null = null;
    if (previousMeaningfulToken && schema?.tables) {
        const lowerPrev = previousMeaningfulToken.toLowerCase();
        const foundTable = schema.tables.find(t => t.toLowerCase() === lowerPrev);
        if (foundTable) {
            matchingTable = foundTable;
        }
    }

    const suggestions: SuggestionItem[] = [];
    const seenLabels = new Set<string>();

    // Priority 1: Keywords
    for (const kw of KEYWORDS) {
        if (kw.toLowerCase().startsWith(lowerToken)) {
            if (!seenLabels.has(kw)) {
                seenLabels.add(kw);
                suggestions.push({ label: kw, kind: "keyword" });
            }
        }
    }

    // Priority 2: Tables
    if (schema?.tables) {
        for (const table of schema.tables) {
            if (table.toLowerCase().startsWith(lowerToken)) {
                if (!seenLabels.has(table)) {
                    seenLabels.add(table);
                    suggestions.push({ label: table, kind: "table" });
                }
            }
        }
    }

    // Priority 3: Columns (from the matching table)
    if (matchingTable && schema?.columns && schema.columns[matchingTable]) {
        for (const col of schema.columns[matchingTable]) {
            if (col.name.toLowerCase().startsWith(lowerToken)) {
                const label = col.name;
                if (!seenLabels.has(label)) {
                    seenLabels.add(label);
                    suggestions.push({
                        label,
                        kind: "column",
                        detail: col.type
                    });
                }
            }
        }
    }

    return suggestions.slice(0, 10);
}
