import { NextResponse } from "next/server";
import { getLLMStatus } from "@/lib/llm";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
  const status = getLLMStatus();
  return NextResponse.json(status);
}
