import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");

    if (!database) {
        return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
    }

    try {
        // We can't use parameters for database names in SHOW TABLES, so we sanitize strictly
        const sanitizedDb = sanitizeIdentifier(database);
        const results: any = await executeQuery(`SHOW TABLES FROM ${sanitizedDb}`);

        const tables = results.map((row: any) => Object.values(row)[0]);

        return NextResponse.json({ success: true, tables });
    } catch (error: any) {
        console.error("Fetch Tables Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch tables" },
            { status: 500 }
        );
    }
}
