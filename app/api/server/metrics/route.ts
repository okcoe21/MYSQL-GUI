import { NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
    try {
        // Fetch a broader set of status variables for performance tracking
        const statusResult: any = await executeQuery(`
            SHOW GLOBAL STATUS WHERE Variable_name IN (
                'Questions', 
                'Threads_connected', 
                'Threads_running', 
                'Bytes_received', 
                'Bytes_sent',
                'Innodb_buffer_pool_pages_total',
                'Innodb_buffer_pool_pages_free',
                'Slow_queries',
                'Uptime'
            )
        `);

        const metrics: Record<string, any> = {};
        statusResult.forEach((row: any) => {
            metrics[row.Variable_name] = row.Value;
        });

        return NextResponse.json({
            success: true,
            timestamp: Date.now(),
            metrics: {
                questions: parseInt(metrics.Questions),
                threads_connected: parseInt(metrics.Threads_connected),
                threads_running: parseInt(metrics.Threads_running),
                bytes_received: parseInt(metrics.Bytes_received),
                bytes_sent: parseInt(metrics.Bytes_sent),
                innodb_total: parseInt(metrics.Innodb_buffer_pool_pages_total),
                innodb_free: parseInt(metrics.Innodb_buffer_pool_pages_free),
                slow_queries: parseInt(metrics.Slow_queries),
                uptime: parseInt(metrics.Uptime)
            }
        });
    } catch (error: any) {
        return NextResponse.json({ success: false, error: error.message }, { status: 500 });
    }
}
