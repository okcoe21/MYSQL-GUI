/**
 * Validates and wraps a MySQL identifier (database, table, column) in backticks.
 * Prevents SQL injection via identifiers by ensuring they only contain valid characters.
 */
export function sanitizeIdentifier(name: string): string {
    if (!name || typeof name !== "string") {
        throw new Error("Invalid identifier scope");
    }

    // MySQL identifiers can contain alphanumeric characters, underscores, and dollar signs.
    // We'll be more strict: only alphanumeric and underscores.
    if (!/^[a-zA-Z0-9_$]+$/.test(name)) {
        throw new Error(`Invalid identifier name: ${name}`);
    }

    return `\`${name}\``;
}

/**
 * Validates if a query might be destructive.
 */
export function isDestructive(query: string): boolean {
    const upper = query.toUpperCase();
    return (
        upper.includes("DROP ") ||
        upper.includes("DELETE ") ||
        upper.includes("TRUNCATE ") ||
        upper.includes("ALTER ")
    );
}
