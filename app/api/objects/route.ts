import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const database = searchParams.get("database");
    const type = searchParams.get("type"); // views, procedures, functions

    if (!database) {
        return NextResponse.json({ success: false, error: "Database is required" }, { status: 400 });
    }

    try {
        const sanitizedDb = sanitizeIdentifier(database);
        let results: any = [];

        if (type === "views") {
            results = await executeQuery(`SELECT TABLE_NAME, VIEW_DEFINITION, IS_UPDATABLE FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?`, [database]);
        } else if (type === "procedures") {
            results = await executeQuery(`SELECT ROUTINE_NAME, ROUTINE_DEFINITION, DATA_TYPE FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'`, [database]);
        } else if (type === "functions") {
            results = await executeQuery(`SELECT ROUTINE_NAME, ROUTINE_DEFINITION, DATA_TYPE FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'FUNCTION'`, [database]);
        } else {
            // Fetch names only for sidebar if no type specified? Or return all?
            // Let's return a summary if no type is specified
            const views: any = await executeQuery(`SELECT TABLE_NAME as name FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?`, [database]);
            const procedures: any = await executeQuery(`SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'`, [database]);
            const functions: any = await executeQuery(`SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'FUNCTION'`, [database]);

            return NextResponse.json({
                success: true,
                views: views.map((v: any) => v.name),
                procedures: procedures.map((p: any) => p.name),
                functions: functions.map((f: any) => f.name)
            });
        }

        return NextResponse.json({ success: true, data: results });
    } catch (error: any) {
        console.error("Fetch Objects Error:", error);
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
