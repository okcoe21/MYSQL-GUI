import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function DELETE(req: NextRequest) {
    try {
        const { database, table, where } = await req.json();

        if (!database || !table || !where || Object.keys(where).length === 0) {
            return NextResponse.json({ success: false, error: "Database, table, and where-clause data are required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        const whereClauses = Object.keys(where).map(col => `${sanitizeIdentifier(col)} = ?`).join(" AND ");
        const values = Object.values(where);

        const query = `DELETE FROM ${sanitizedDb}.${sanitizedTable} WHERE ${whereClauses} LIMIT 1`;

        const result: any = await executeQuery(query, values);

        return NextResponse.json({
            success: true,
            affectedRows: result.affectedRows
        });
    } catch (error: any) {
        console.error("Delete Data Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to delete data" },
            { status: 500 }
        );
    }
}
