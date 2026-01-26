import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");

    if (!database) {
        return NextResponse.json({ success: false, error: "Database is required" }, { status: 400 });
    }

    try {
        const query = `
            SELECT 
                TABLE_NAME, 
                COLUMN_NAME, 
                CONSTRAINT_NAME, 
                REFERENCED_TABLE_NAME, 
                REFERENCED_COLUMN_NAME 
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE 
            WHERE TABLE_SCHEMA = ? 
            AND REFERENCED_TABLE_NAME IS NOT NULL
        `;

        const relations: any = await executeQuery(query, [database]);

        return NextResponse.json({ success: true, relations });
    } catch (error: any) {
        console.error("Fetch Relations Error:", error);
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
