import { NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
    try {
        const processes: any = await executeQuery("SHOW FULL PROCESSLIST");

        // Also get some basic status variables for health monitoring
        const statusResult: any = await executeQuery("SHOW GLOBAL STATUS WHERE Variable_name IN ('Uptime', 'Threads_connected', 'Threads_running', 'Questions', 'Slow_queries')");
        const status: Record<string, string> = {};
        statusResult.forEach((row: any) => {
            status[row.Variable_name] = row.Value;
        });

        return NextResponse.json({
            success: true,
            processes,
            status: {
                uptime: parseInt(status.Uptime),
                threads_connected: parseInt(status.Threads_connected),
                threads_running: parseInt(status.Threads_running),
                queries: parseInt(status.Questions),
                slow_queries: parseInt(status.Slow_queries)
            }
        });
    } catch (error: any) {
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
