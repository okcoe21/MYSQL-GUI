import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function PATCH(req: NextRequest) {
    try {
        const { database, table, column, value, where } = await req.json();

        if (!database || !table || !column || where === undefined) {
            return NextResponse.json({ success: false, error: "Missing required parameters" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);
        const sanitizedCol = sanitizeIdentifier(column);

        // Build WHERE clause based on the original row data
        const whereClause = Object.keys(where)
            .map(k => `${sanitizeIdentifier(k)} = ?`)
            .join(" AND ");
        const whereValues = Object.values(where);

        const query = `UPDATE ${sanitizedDb}.${sanitizedTable} SET ${sanitizedCol} = ? WHERE ${whereClause} LIMIT 1`;
        const params = [value, ...whereValues];

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
