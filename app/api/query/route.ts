import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { isDestructive } from "@/lib/sanitize";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
    try {
        const { query, database, confirmed } = await req.json();

        if (!query) {
            return NextResponse.json({ success: false, error: "Query is required" }, { status: 400 });
        }

        // 1. Check for destructive queries if not confirmed
        if (isDestructive(query) && !confirmed) {
            return NextResponse.json({
                success: false,
                error: "DESTRUCTIVE_QUERY",
                message: "This query contains potentially destructive operations (DROP, DELETE, TRUNCATE, ALTER). Are you sure you want to proceed?",
            });
        }

        // 2. Execute query
        // mysql2/promise `execute` prepared statements doesn't support multi-statements by default.
        // If the user uses `query` (as our executeQuery does), it might support them if enabled in connection.
        // We'll stick to `execute` if possible, but for raw SQL we use `query`.
        const results: any = await executeQuery(query, [], database);

        // results can be:
        // - An array of rows (for SELECT)
        // - An object with info (for INSERT/UPDATE/DELETE)

        let processedData = results;
        let columns: string[] = [];
        let affectedRows = null;

        if (Array.isArray(results)) {
            if (results.length > 0 && typeof results[0] === "object") {
                columns = Object.keys(results[0]);
            }
        } else if (results && typeof results === "object") {
            affectedRows = results.affectedRows;
            processedData = null;
        }

        return NextResponse.json({
            success: true,
            data: processedData,
            columns,
            affectedRows
        });
    } catch (error: any) {
        console.error("SQL Query Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to execute SQL query" },
            { status: 500 }
        );
    }
}
