import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");
    const table = searchParams.get("table");

    if (!database || !table) {
        return NextResponse.json({ success: false, error: "Database and Table names are required" }, { status: 400 });
    }

    try {
        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        // Fetch data (limit 100)
        const results: any = await executeQuery(`SELECT * FROM ${sanitizedDb}.${sanitizedTable} LIMIT 100`);

        // Also fetch column names if results are empty or just to be sure
        // mysql2 returns rows as objects, so keys are column names
        const columns = results.length > 0 ? Object.keys(results[0]) : [];

        return NextResponse.json({ success: true, data: results, columns });
    } catch (error: any) {
        console.error("Fetch Data Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch table data" },
            { status: 500 }
        );
    }
}
