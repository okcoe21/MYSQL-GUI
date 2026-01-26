import { NextRequest, NextResponse } from "next/server";
import { executeQuery } from "@/lib/db";
import { sanitizeIdentifier } from "@/lib/sanitize";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function POST(req: NextRequest) {
    try {
        const { name } = await req.json();

        if (!name) {
            return NextResponse.json({ success: false, error: "Database name is required" }, { status: 400 });
        }

        const sanitizedDb = sanitizeIdentifier(name);
        const query = `CREATE DATABASE ${sanitizedDb}`;

        await executeQuery(query);

        return NextResponse.json({ success: true });
    } catch (error: any) {
        console.error("Create Database Error:", error);
        return NextResponse.json(
            { success: false, error: error.message || "Failed to create database" },
            { status: 500 }
        );
    }
}
