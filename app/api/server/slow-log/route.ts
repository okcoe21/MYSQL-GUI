import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
    try {
        // Check if slow query logging to table is enabled
        const checkRes: any = await executeQuery(`SHOW VARIABLES LIKE 'log_output'`);
        const logOutput = checkRes.find((r: any) => r.Variable_name === 'log_output')?.Value || "";

        if (!logOutput.includes("TABLE")) {
            return NextResponse.json({
                success: false,
                error: "Slow query logging to table is not enabled. Run: SET GLOBAL log_output = 'TABLE'; SET GLOBAL slow_query_log = 'ON';",
                setupRequired: true
            });
        }

        const logs: any = await executeQuery(`SELECT * FROM mysql.slow_log ORDER BY start_time DESC LIMIT 100`);

        return NextResponse.json({ success: true, logs });
    } catch (error: any) {
        console.error("Slow Log Error:", error);
        return NextResponse.json({
            success: false,
            error: error.code === "ER_TABLEACCESS_DENIED_ERROR"
                ? "Insufficient privileges to read 'mysql.slow_log' table."
                : error.message
        }, { status: 500 });
    }
}
