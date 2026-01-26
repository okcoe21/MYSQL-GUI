import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

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

        // Fetch structure
        const results: any = await executeQuery(`DESCRIBE ${sanitizedDb}.${sanitizedTable}`);

        return NextResponse.json({ success: true, structure: results });
    } catch (error: any) {
        console.error("Fetch Structure Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to fetch table structure" },
            { status: 500 }
        );
    }
}
