import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

const VALID_TYPES = [
    "INT", "TINYINT", "SMALLINT", "MEDIUMINT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE", "REAL", "BIT", "BOOLEAN", "SERIAL",
    "DATE", "DATETIME", "TIMESTAMP", "TIME", "YEAR",
    "CHAR", "VARCHAR", "TINYTEXT", "TEXT", "MEDIUMTEXT", "LONGTEXT", "BINARY", "VARBINARY", "TINYBLOB", "BLOB", "MEDIUMBLOB", "LONGBLOB", "ENUM", "SET", "JSON"
];

export async function POST(req: NextRequest) {
    try {
        const { database, table, columns } = await req.json();

        if (!database || !table || !columns || !Array.isArray(columns) || columns.length === 0) {
            return NextResponse.json({ success: false, error: "Database, Table, and Columns are required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        let primaryKeys: string[] = [];

        const colDefs = columns.map(col => {
            const name = sanitizeIdentifier(col.name);
            const type = col.type.toUpperCase();

            if (!VALID_TYPES.includes(type)) {
                throw new Error(`Invalid data type: [${type}]`);
            }

            const length = col.length ? `(${col.length})` : "";
            const isNull = col.isNull ? "NULL" : "NOT NULL";
            const autoInc = col.isAutoIncrement ? "AUTO_INCREMENT" : "";

            if (col.isPrimary) {
                primaryKeys.push(name);
            }

            return `${name} ${type}${length} ${isNull} ${autoInc}`.trim();
        });

        if (primaryKeys.length > 0) {
            colDefs.push(`PRIMARY KEY (${primaryKeys.join(", ")})`);
        }

        const query = `CREATE TABLE ${sanitizedDb}.${sanitizedTable} (${colDefs.join(", ")})`;

        await executeQuery(query);

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Create Table Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to create table" },
            { status: 500 }
        );
    }
}
