import { NextRequest, NextResponse } from "next/server";
import { getLLMStatus, llmComplete } from "@/lib/llm";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function POST(req: NextRequest) {
  try {
    const status = getLLMStatus();
    if (!status.available) {
      return NextResponse.json({ error: "No LLM provider configured" }, { status: 501 });
    }

    const body = await req.json().catch(() => ({}));
    const { prompt, schema } = body;

    if (!prompt || !prompt.trim()) {
      return NextResponse.json({ error: "Prompt is required" }, { status: 400 });
    }

    if (!schema || !schema.tables || schema.tables.length === 0) {
      return NextResponse.json({ error: "Schema is required" }, { status: 400 });
    }

    // Build system prompt
    const systemPrompt = `You are a MySQL expert. Given a database schema and a plain-English request, generate a single valid MySQL query.

Rules:
- Output ONLY the raw SQL — no explanation, no markdown, no code fences, no preamble, no trailing semicolons
- Use only the tables and columns that exist in the provided schema
- Prefer SELECT unless the request clearly implies INSERT, UPDATE, or DELETE
- Always quote identifiers with backticks
- If the request is ambiguous, generate the most reasonable SELECT query
- If the request cannot be fulfilled with the given schema, output exactly: -- Cannot generate: [reason]`;

    // Build schema description string
    let schemaText = "Database schema:\n";
    for (const table of schema.tables) {
      const cols = schema.columns[table] || [];
      const colDetails = cols.map((c: any) => `${c.name} (${c.type})`).join(", ");
      schemaText += `\`${table}\`: ${colDetails}\n`;
    }

    const userMessage = `${schemaText}\nRequest: ${prompt}`;

    const result = await llmComplete(systemPrompt, userMessage, 300);

    let sql = result.text.trim();
    
    // Clean up response if the LLM outputted code blocks anyway
    if (sql.startsWith("```")) {
      sql = sql.replace(/^```(sql)?\s*/i, "").replace(/\s*```$/, "").trim();
    }

    return NextResponse.json({
      sql,
      provider: result.provider,
    });
  } catch (error: any) {
    console.error("NLQ Route Error:", error);
    return NextResponse.json(
      { error: error.message || "Failed to generate query" },
      { status: 502 }
    );
  }
}
