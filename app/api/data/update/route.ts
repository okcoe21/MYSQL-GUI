import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function PATCH(req: NextRequest) {
    try {
        const { database, table, column, value, where } = await req.json();

        if (!database || !table || !column || where === undefined) {
            return NextResponse.json({ success: false, error: "Missing required parameters" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);
        const sanitizedCol = sanitizeIdentifier(column);

        // Fetch table structure to know column type
        const structureQuery = `DESCRIBE ${sanitizedDb}.${sanitizedTable}`;
        const structure = await executeQuery(structureQuery) as any[];
        const colInfo = structure.find(s => s.Field === column);

        let processedValue = value;
        if (colInfo && (colInfo.Type.includes('datetime') || colInfo.Type.includes('timestamp')) && typeof value === 'string') {
            // Try to parse ISO string and convert to MySQL format
            const date = new Date(value);
            if (!isNaN(date.getTime())) {
                processedValue = date.toISOString().slice(0, 19);
            }
        }

        // Build WHERE clause based on the original row data
        const whereClause = Object.keys(where)
            .map(k => `${sanitizeIdentifier(k)} = ?`)
            .join(" AND ");
        const whereValues = Object.values(where);

        const query = `UPDATE ${sanitizedDb}.${sanitizedTable} SET ${sanitizedCol} = ? WHERE ${whereClause} LIMIT 1`;
        const params = [processedValue, ...whereValues];

        await executeQuery(query, params);

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Update Data Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to update table data" },
            { status: 500 }
        );
    }
}
