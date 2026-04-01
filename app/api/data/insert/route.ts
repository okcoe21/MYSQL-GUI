import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function POST(req: NextRequest) {
    try {
        const { database, table, data } = await req.json();

        if (!database || !table || !data) {
            return NextResponse.json({ success: false, error: "Database, table, and data are required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        // Fetch table structure to know column types
        const structureQuery = `DESCRIBE ${sanitizedDb}.${sanitizedTable}`;
        const structure = await executeQuery(structureQuery) as any[];

        const columns = Object.keys(data);
        const sanitizedColumns = columns.map(col => sanitizeIdentifier(col));
        const placeholders = columns.map(() => "?").join(", ");
        const values = columns.map(col => {
            const value = data[col];
            const colInfo = structure.find(s => s.Field === col);
            if (colInfo && (colInfo.Type.includes('datetime') || colInfo.Type.includes('timestamp')) && typeof value === 'string') {
                // Try to parse ISO string and convert to MySQL format
                const date = new Date(value);
                if (!isNaN(date.getTime())) {
                    return date.toISOString().slice(0, 19);
                }
            }
            return value;
        });

        const query = `INSERT INTO ${sanitizedDb}.${sanitizedTable} (${sanitizedColumns.join(", ")}) VALUES (${placeholders})`;

        const result: any = await executeQuery(query, values);

        return NextResponse.json({
            success: true,
            insertId: result.insertId,
            affectedRows: result.affectedRows
        });
    } catch (error: any) {
        console.error("Insert Data Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to insert data" },
            { status: 500 }
        );
    }
}
