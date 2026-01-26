import { NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
    try {
        // Requires SELECT on mysql.user table - normally provided to the user connecting
        const users: any = await executeQuery("SELECT user, host, authentication_string, account_locked from mysql.user");
        return NextResponse.json({ success: true, users });
    } catch (error: any) {
        // Fallback for users who can't access mysql.user
        try {
            const userSelf: any = await executeQuery("SELECT USER()");
            return NextResponse.json({
                success: true,
                users: [{ user: userSelf[0]['USER()'].split('@')[0], host: userSelf[0]['USER()'].split('@')[1] }],
                note: "Limited visibility: could not access mysql.user table"
            });
        } catch (innerError: any) {
            return NextResponse.json({ success: false, error: error.message }, { status: 500 });
        }
    }
}
