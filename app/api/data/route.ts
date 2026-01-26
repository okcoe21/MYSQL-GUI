import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");
    const table = searchParams.get("table");
    const limit = parseInt(searchParams.get("limit") || "100");
    const offset = parseInt(searchParams.get("offset") || "0");
    const sortCol = searchParams.get("sortCol");
    const sortOrder = searchParams.get("sortOrder") === "DESC" ? "DESC" : "ASC";

    if (!database || !table) {
        return NextResponse.json({ success: false, error: "Database and Table names are required" }, { status: 400 });
    }

    try {
        const sanitizedDb = sanitizeIdentifier(database);
        const sanitizedTable = sanitizeIdentifier(table);

        let query = `SELECT * FROM ${sanitizedDb}.${sanitizedTable}`;
        const params: any[] = [];

        if (sortCol) {
            const sanitizedSortCol = sanitizeIdentifier(sortCol);
            query += ` ORDER BY ${sanitizedSortCol} ${sortOrder}`;
        }

        query += ` LIMIT ? OFFSET ?`;
        params.push(limit, offset);

        // Fetch data
        const results: any = await executeQuery(query, params);

        // Fetch total count for pagination
        const countResult: any = await executeQuery(`SELECT COUNT(*) as total FROM ${sanitizedDb}.${sanitizedTable}`);
        const total = countResult[0].total;

        // Also fetch column names if results are empty or just to be sure
        const columns = results.length > 0 ? Object.keys(results[0]) : [];

        return NextResponse.json({
            success: true,
            data: results,
            columns,
            pagination: {
                total,
                limit,
                offset
            }
        });
    } catch (error: any) {
        console.error("Fetch Data Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch table data" },
            { status: 500 }
        );
    }
}
