/**
 * SQL Query Explainer and Parser Module
 * A pure TypeScript parser that explains SQL queries in plain English.
 */

export function explainQuery(sql: string): string {
    if (!sql || !sql.trim()) {
        return "This query could not be parsed. Check the syntax and try again.";
    }

    try {
        let hasSubquery = false;

        // Helper to replace parenthesized SELECT subqueries with [subquery] from innermost out
        function processSubqueries(input: string): string {
            let result = input;
            let changed = true;
            while (changed) {
                changed = false;
                let start = -1;
                for (let i = 0; i < result.length; i++) {
                    if (result[i] === '(') {
                        start = i;
                    } else if (result[i] === ')') {
                        if (start !== -1) {
                            const content = result.substring(start + 1, i);
                            if (/\bSELECT\b/i.test(content)) {
                                hasSubquery = true;
                                result = result.substring(0, start + 1) + "[subquery]" + result.substring(i);
                                changed = true;
                                break;
                            }
                            start = -1;
                        }
                    }
                }
            }
            return result;
        }

        // Normalize input: trim, collapse whitespace, strip comments
        let normalized = sql
            .replace(/\/\*[\s\S]*?\*\//g, "") // Block comments
            .replace(/--[^\r\n]*/g, "")      // Line comments
            .trim();

        // Process subqueries
        normalized = processSubqueries(normalized);
        normalized = normalized.replace(/\s+/g, " ");

        // Tokenize using regex on whitespace and punctuation boundaries
        // Matches string literals, multi-char operators, words/numbers/identifiers (including dot and *), and individual punctuation
        const rawTokens = normalized.match(/'(?:''|[^'])*'|"[^"]*"|[a-zA-Z0-9_\*\.]+|!=|<>|>=|<=|[=><,()!]/g) || [];
        const tokens = rawTokens.map(t => ({ text: t, upper: t.toUpperCase() }));

        if (tokens.length === 0) {
            return "This query could not be parsed. Check the syntax and try again.";
        }

        const firstWord = tokens[0].upper;

        let explanation = "";

        if (firstWord === "SELECT") {
            explanation = handleSelect(tokens);
        } else if (firstWord === "INSERT") {
            explanation = handleInsert(tokens);
        } else if (firstWord === "UPDATE") {
            explanation = handleUpdate(tokens);
        } else if (firstWord === "DELETE") {
            explanation = handleDelete(tokens);
        } else if (firstWord === "CREATE" && tokens[1]?.upper === "TABLE") {
            explanation = handleCreateTable(tokens);
        } else if (firstWord === "DROP") {
            explanation = handleDrop(tokens);
        } else if (firstWord === "ALTER" && tokens[1]?.upper === "TABLE") {
            explanation = handleAlterTable(tokens);
        } else {
            return "This query could not be parsed. Check the syntax and try again.";
        }

        if (hasSubquery) {
            explanation += " The WHERE condition includes a subquery.";
        }

        return explanation;

    } catch (err) {
        return "This query could not be parsed. Check the syntax and try again.";
    }
}

// Token helper: joins array of tokens back into string
function tokensToString(tokens: { text: string; upper: string }[]): string {
    return tokens.map(t => t.text).join(" ");
}

// Splits tokens by commas at parenthesis depth 0
function splitByComma(tokens: { text: string; upper: string }[]): { text: string; upper: string }[][] {
    const parts: { text: string; upper: string }[][] = [];
    let current: { text: string; upper: string }[] = [];
    let depth = 0;
    for (const t of tokens) {
        if (t.text === '(') depth++;
        else if (t.text === ')') depth--;

        if (t.text === ',' && depth === 0) {
            parts.push(current);
            current = [];
        } else {
            current.push(t);
        }
    }
    if (current.length > 0) {
        parts.push(current);
    }
    return parts;
}

// Formats a list of strings: "A", "A and B", "A, B and C"
function formatList(items: string[]): string {
    if (items.length === 0) return "";
    if (items.length === 1) return items[0];
    if (items.length === 2) return `${items[0]} and ${items[1]}`;
    return `${items.slice(0, -1).join(", ")} and ${items[items.length - 1]}`;
}

// Formats SQL operators to plain English in condition clauses
function formatCondition(tokens: { text: string; upper: string }[]): string {
    const result: string[] = [];
    for (let i = 0; i < tokens.length; i++) {
        const t = tokens[i];
        const next = tokens[i + 1];
        const next2 = tokens[i + 2];

        if (t.upper === 'IS' && next?.upper === 'NOT' && next2?.upper === 'NULL') {
            result.push('is not empty');
            i += 2;
        } else if (t.upper === 'IS' && next?.upper === 'NULL') {
            result.push('is empty');
            i += 1;
        } else if (t.upper === '=') {
            result.push('equals');
        } else if (t.upper === '!=' || t.upper === '<>') {
            result.push('not equal to');
        } else if (t.upper === '>=') {
            result.push('at least');
        } else if (t.upper === '<=') {
            result.push('at most');
        } else if (t.upper === '>') {
            result.push('greater than');
        } else if (t.upper === '<') {
            result.push('less than');
        } else if (t.upper === 'LIKE') {
            result.push('matches pattern');
        } else if (t.upper === 'IN') {
            result.push('is one of');
        } else if (t.upper === 'BETWEEN') {
            result.push('is between');
        } else {
            result.push(t.text);
        }
    }
    return result.join(" ");
}

interface ClauseBoundary {
    name: string;
    startIndex: number;
    endIndex: number;
    extra?: any;
}

function handleSelect(tokens: { text: string; upper: string }[]): string {
    let fromIndex = -1;
    let whereIndex = -1;
    let groupByIndex = -1;
    let havingIndex = -1;
    let orderByIndex = -1;
    let limitIndex = -1;
    const joinIndexes: { type: string; index: number }[] = [];

    for (let i = 0; i < tokens.length; i++) {
        const t = tokens[i];
        const next = tokens[i + 1];

        if (t.upper === 'FROM') {
            fromIndex = i;
        } else if (t.upper === 'WHERE') {
            whereIndex = i;
        } else if (t.upper === 'GROUP' && next?.upper === 'BY') {
            groupByIndex = i;
        } else if (t.upper === 'HAVING') {
            havingIndex = i;
        } else if (t.upper === 'ORDER' && next?.upper === 'BY') {
            orderByIndex = i;
        } else if (t.upper === 'LIMIT') {
            limitIndex = i;
        } else if (t.upper === 'JOIN') {
            joinIndexes.push({ type: 'joined', index: i });
        } else if (t.upper === 'INNER' && next?.upper === 'JOIN') {
            joinIndexes.push({ type: 'inner joined', index: i });
        } else if (t.upper === 'CROSS' && next?.upper === 'JOIN') {
            joinIndexes.push({ type: 'cross joined', index: i });
        } else if (t.upper === 'LEFT' && next?.upper === 'JOIN') {
            joinIndexes.push({ type: 'left joined', index: i });
        } else if (t.upper === 'LEFT' && next?.upper === 'OUTER' && tokens[i + 2]?.upper === 'JOIN') {
            joinIndexes.push({ type: 'left joined', index: i });
        } else if (t.upper === 'RIGHT' && next?.upper === 'JOIN') {
            joinIndexes.push({ type: 'right joined', index: i });
        } else if (t.upper === 'RIGHT' && next?.upper === 'OUTER' && tokens[i + 2]?.upper === 'JOIN') {
            joinIndexes.push({ type: 'right joined', index: i });
        }
    }

    const boundaries: ClauseBoundary[] = [
        { name: 'SELECT', startIndex: 0, endIndex: 1 }
    ];
    if (fromIndex !== -1) {
        boundaries.push({ name: 'FROM', startIndex: fromIndex, endIndex: fromIndex + 1 });
    }
    for (const j of joinIndexes) {
        let len = 1;
        const next = tokens[j.index + 1]?.upper;
        const next2 = tokens[j.index + 2]?.upper;
        if (j.type !== 'joined') {
            if (next === 'OUTER' && next2 === 'JOIN') {
                len = 3;
            } else {
                len = 2;
            }
        }
        boundaries.push({ name: 'JOIN', startIndex: j.index, endIndex: j.index + len, extra: j.type });
    }
    if (whereIndex !== -1) {
        boundaries.push({ name: 'WHERE', startIndex: whereIndex, endIndex: whereIndex + 1 });
    }
    if (groupByIndex !== -1) {
        boundaries.push({ name: 'GROUP BY', startIndex: groupByIndex, endIndex: groupByIndex + 2 });
    }
    if (havingIndex !== -1) {
        boundaries.push({ name: 'HAVING', startIndex: havingIndex, endIndex: havingIndex + 1 });
    }
    if (orderByIndex !== -1) {
        boundaries.push({ name: 'ORDER BY', startIndex: orderByIndex, endIndex: orderByIndex + 2 });
    }
    if (limitIndex !== -1) {
        boundaries.push({ name: 'LIMIT', startIndex: limitIndex, endIndex: limitIndex + 1 });
    }

    boundaries.sort((a, b) => a.startIndex - b.startIndex);

    const clauseTokens: { [key: string]: { text: string; upper: string }[] } = {};
    const joinClauses: { type: string; tokens: { text: string; upper: string }[] }[] = [];

    for (let idx = 0; idx < boundaries.length; idx++) {
        const current = boundaries[idx];
        const nextStart = (idx + 1 < boundaries.length) ? boundaries[idx + 1].startIndex : tokens.length;
        const t = tokens.slice(current.endIndex, nextStart);
        if (current.name === 'JOIN') {
            joinClauses.push({ type: current.extra, tokens: t });
        } else {
            clauseTokens[current.name] = t;
        }
    }

    const parts: string[] = [];

    // Columns: "Fetches X, Y and Z" / "Fetches all columns"
    const selectT = clauseTokens['SELECT'] || [];
    const colNames = selectT.length > 0 ? splitByComma(selectT).map(tokensToString) : [];
    let formattedCols = "";
    if (colNames.length === 1 && colNames[0] === "*") {
        formattedCols = "all columns";
    } else {
        formattedCols = formatList(colNames);
    }

    let selectPart = `Fetches ${formattedCols}`;

    // Table: "from the {table} table"
    const fromT = clauseTokens['FROM'] || [];
    if (fromT.length > 0) {
        // First token is table
        const table = fromT[0].text;
        selectPart += ` from the ${table} table`;
    }
    parts.push(selectPart);

    // JOINs: detect INNER/LEFT/RIGHT/CROSS JOIN
    if (joinClauses.length > 0) {
        const joinDescriptions = joinClauses.map(jc => {
            const table = jc.tokens[0]?.text || "";
            const onIndex = jc.tokens.findIndex(t => t.upper === "ON");
            if (onIndex !== -1 && jc.type !== "cross joined") {
                const condT = jc.tokens.slice(onIndex + 1);
                const condition = formatCondition(condT);
                return `${jc.type} with ${table} on ${condition}`;
            }
            return `${jc.type} with ${table}`;
        });
        parts.push(formatList(joinDescriptions));
    }

    // WHERE
    const whereT = clauseTokens['WHERE'] || [];
    if (whereT.length > 0) {
        parts.push(`filtered where ${formatCondition(whereT)}`);
    }

    // GROUP BY
    const groupByT = clauseTokens['GROUP BY'] || [];
    if (groupByT.length > 0) {
        parts.push(`grouped by ${tokensToString(groupByT)}`);
    }

    // HAVING
    const havingT = clauseTokens['HAVING'] || [];
    if (havingT.length > 0) {
        parts.push(`having ${formatCondition(havingT)}`);
    }

    // ORDER BY
    const orderByT = clauseTokens['ORDER BY'] || [];
    if (orderByT.length > 0) {
        const orderByParts = splitByComma(orderByT).map(partTokens => {
            const lastToken = partTokens[partTokens.length - 1]?.upper;
            let dir = "ascending";
            let colTokens = partTokens;
            if (lastToken === "DESC") {
                dir = "descending";
                colTokens = partTokens.slice(0, -1);
            } else if (lastToken === "ASC") {
                dir = "ascending";
                colTokens = partTokens.slice(0, -1);
            }
            return `${tokensToString(colTokens)} ${dir}`;
        });
        parts.push(`sorted by ${orderByParts.join(" then ")}`);
    }

    // LIMIT
    const limitT = clauseTokens['LIMIT'] || [];
    if (limitT.length > 0) {
        parts.push(`limited to ${limitT[0].text} rows`);
    }

    return parts.join(", ") + ".";
}

function handleInsert(tokens: { text: string; upper: string }[]): string {
    let table = "";
    const intoIndex = tokens.findIndex(t => t.upper === "INTO");
    if (intoIndex !== -1 && intoIndex + 1 < tokens.length) {
        table = tokens[intoIndex + 1].text;
    } else {
        table = tokens[1]?.text || "";
    }

    const valuesIndex = tokens.findIndex(t => t.upper === "VALUES");

    let cols: string[] = [];
    let vals: string[] = [];

    function extractParenthesizedItems(startIdx: number, endIdx: number): string[] {
        const openIdx = tokens.findIndex((t, idx) => idx >= startIdx && idx < endIdx && t.text === '(');
        if (openIdx === -1) return [];
        let depth = 1;
        let closeIdx = -1;
        for (let i = openIdx + 1; i < endIdx; i++) {
            if (tokens[i].text === '(') depth++;
            if (tokens[i].text === ')') {
                depth--;
                if (depth === 0) {
                    closeIdx = i;
                    break;
                }
            }
        }
        if (closeIdx === -1) return [];
        const innerTokens = tokens.slice(openIdx + 1, closeIdx);
        return splitByComma(innerTokens).map(tokensToString);
    }

    if (valuesIndex !== -1) {
        const tableTokenIndex = tokens.findIndex(t => t.text === table);
        cols = extractParenthesizedItems(tableTokenIndex !== -1 ? tableTokenIndex + 1 : 0, valuesIndex);
        vals = extractParenthesizedItems(valuesIndex + 1, tokens.length);
    }

    if (cols.length > 0 && vals.length > 0 && cols.length === vals.length) {
        const assignments = cols.map((col, idx) => `${col} to ${vals[idx]}`).join(", ");
        return `Inserts a new row into ${table} setting ${assignments}.`;
    }

    return `Inserts a new row into ${table} with the provided values.`;
}

function handleUpdate(tokens: { text: string; upper: string }[]): string {
    const table = tokens[1]?.text || "";
    const setIndex = tokens.findIndex(t => t.upper === "SET");
    const whereIndex = tokens.findIndex(t => t.upper === "WHERE");

    if (setIndex === -1) {
        return "This query could not be parsed. Check the syntax and try again.";
    }

    const assignmentTokens = tokens.slice(setIndex + 1, whereIndex !== -1 ? whereIndex : tokens.length);
    const assignmentParts = splitByComma(assignmentTokens).map(partTokens => {
        const eqIdx = partTokens.findIndex(t => t.text === "=");
        if (eqIdx !== -1) {
            const col = tokensToString(partTokens.slice(0, eqIdx));
            const val = tokensToString(partTokens.slice(eqIdx + 1));
            return `${col} to ${val}`;
        }
        return tokensToString(partTokens);
    });
    const assignmentsString = assignmentParts.join(", ");

    if (whereIndex !== -1) {
        const whereTokens = tokens.slice(whereIndex + 1);
        const condition = formatCondition(whereTokens);
        return `Updates rows in ${table}, setting ${assignmentsString}, where ${condition}.`;
    }

    return `⚠ Updates ALL rows in ${table} — no WHERE clause detected. Updates rows in ${table}, setting ${assignmentsString}.`;
}

function handleDelete(tokens: { text: string; upper: string }[]): string {
    const fromIndex = tokens.findIndex(t => t.upper === "FROM");
    const table = fromIndex !== -1 ? tokens[fromIndex + 1]?.text || "" : tokens[1]?.text || "";
    const whereIndex = tokens.findIndex(t => t.upper === "WHERE");

    if (whereIndex !== -1) {
        const whereTokens = tokens.slice(whereIndex + 1);
        const condition = formatCondition(whereTokens);
        return `Deletes rows from ${table} where ${condition}.`;
    }

    return `⚠ Deletes ALL rows from ${table} — no WHERE clause detected. Deletes rows from ${table}.`;
}

function handleCreateTable(tokens: { text: string; upper: string }[]): string {
    const table = tokens[2]?.text || "";
    const openIdx = tokens.findIndex(t => t.text === "(");
    if (openIdx === -1) {
        return `Creates a new table named ${table}.`;
    }

    const innerTokens = tokens.slice(openIdx + 1, tokens.length - 1);
    const colParts = splitByComma(innerTokens);
    const colDescriptions: string[] = [];

    const constraints = new Set([
        "NOT", "NULL", "PRIMARY", "KEY", "AUTO_INCREMENT", "AUTOINCREMENT",
        "UNIQUE", "DEFAULT", "REFERENCES", "CHECK", "FOREIGN"
    ]);

    for (const part of colParts) {
        if (part.length === 0) continue;
        const colName = part[0].text;
        
        // Skip table constraints like CONSTRAINT pk PRIMARY KEY (id)
        if (colName.toUpperCase() === "CONSTRAINT" || colName.toUpperCase() === "PRIMARY" || colName.toUpperCase() === "FOREIGN" || colName.toUpperCase() === "KEY") {
            continue;
        }

        const typeTokens: string[] = [];
        for (let j = 1; j < part.length; j++) {
            if (constraints.has(part[j].upper)) {
                break;
            }
            typeTokens.push(part[j].text);
        }

        let typeStr = typeTokens.join(" ");
        typeStr = typeStr.replace(/\s*\(\s*/g, "(").replace(/\s*\)\s*/g, ")");
        if (typeStr) {
            colDescriptions.push(`${colName} (${typeStr})`);
        } else {
            colDescriptions.push(colName);
        }
    }

    return `Creates a new table named ${table} with ${colDescriptions.length} columns: ${colDescriptions.join(", ")}.`;
}

function handleDrop(tokens: { text: string; upper: string }[]): string {
    const rawType = tokens[1]?.upper || "";
    const name = tokens[2]?.text || "";
    const type = rawType === "TABLE" ? "table" : (rawType === "DATABASE" || rawType === "SCHEMA") ? "database" : rawType.toLowerCase();
    return `⚠ Permanently drops the ${type} named ${name}. This cannot be undone.`;
}

function handleAlterTable(tokens: { text: string; upper: string }[]): string {
    const table = tokens[2]?.text || "";
    
    // Find action keyword
    let addIndex = -1;
    let dropIndex = -1;
    let modifyIndex = -1;
    let changeIndex = -1;

    for (let i = 3; i < tokens.length; i++) {
        const upper = tokens[i].upper;
        if (upper === "ADD") addIndex = i;
        else if (upper === "DROP") dropIndex = i;
        else if (upper === "MODIFY") modifyIndex = i;
        else if (upper === "CHANGE") changeIndex = i;
    }

    if (addIndex !== -1) {
        let colStart = addIndex + 1;
        if (tokens[colStart]?.upper === "COLUMN") colStart++;
        const col = tokens[colStart]?.text || "";
        
        const typeTokens: string[] = [];
        for (let j = colStart + 1; j < tokens.length; j++) {
            typeTokens.push(tokens[j].text);
        }
        let typeStr = typeTokens.join(" ");
        typeStr = typeStr.replace(/\s*\(\s*/g, "(").replace(/\s*\)\s*/g, ")");
        return `Adds a ${typeStr || "new"} column named ${col} to ${table}.`;
    }

    if (dropIndex !== -1) {
        let colStart = dropIndex + 1;
        if (tokens[colStart]?.upper === "COLUMN") colStart++;
        const col = tokens[colStart]?.text || "";
        return `Removes the column ${col} from ${table}.`;
    }

    if (modifyIndex !== -1) {
        let colStart = modifyIndex + 1;
        if (tokens[colStart]?.upper === "COLUMN") colStart++;
        const col = tokens[colStart]?.text || "";
        return `Changes the ${col} column in ${table}.`;
    }

    if (changeIndex !== -1) {
        let colStart = changeIndex + 1;
        if (tokens[colStart]?.upper === "COLUMN") colStart++;
        const col = tokens[colStart]?.text || "";
        return `Changes the ${col} column in ${table}.`;
    }

    return "This query could not be parsed. Check the syntax and try again.";
}
