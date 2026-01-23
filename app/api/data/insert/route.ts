import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
    try {
        const { database, table, data } = await req.json();

        if (!database || !table || !data) {
            return NextResponse.json({ success: false, error: "Database, table, and data are required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        const columns = Object.keys(data).map(col => sanitizeIdentifier(col)).join(", ");
        const placeholders = Object.keys(data).map(() => "?").join(", ");
        const values = Object.values(data);

        const query = `INSERT INTO ${sanitizedDb}.${sanitizedTable} (${columns}) VALUES (${placeholders})`;

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
