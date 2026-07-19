export type LLMProvider = 'anthropic' | 'openai' | 'gemini' | 'ollama';

export interface LLMResult {
  text: string;
  provider: LLMProvider;
}

export async function llmComplete(prompt: string, schema: any, config: any): Promise<LLMResult> {
  if (!config || !config.provider) {
    throw new Error('No LLM provider configured');
  }

  const { provider, key, ollama_host, ollama_model } = config;
  const model = provider === 'anthropic' ? 'claude-haiku-3-5' :
                provider === 'openai' ? 'gpt-4o-mini' :
                provider === 'gemini' ? 'gemini-1.5-flash' :
                ollama_model || 'llama3';

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
  if (schema && schema.tables) {
    for (const table of schema.tables) {
      const cols = schema.columns[table] || [];
      const colDetails = cols.map((c: any) => `${c.name} (${c.type})`).join(", ");
      schemaText += `\`${table}\`: ${colDetails}\n`;
    }
  }

  const userMessage = `${schemaText}\nRequest: ${prompt}`;
  const maxTokens = 300;

  try {
    if (provider === 'anthropic') {
      const apiKey = key || '';
      const response = await fetch('https://api.anthropic.com/v1/messages', {
        method: 'POST',
        headers: {
          'x-api-key': apiKey,
          'anthropic-version': '2023-06-01',
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          model,
          max_tokens: maxTokens,
          system: systemPrompt,
          messages: [{ role: 'user', content: userMessage }],
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`[anthropic] ${response.status}: ${errorText}`);
      }

      const data = await response.json();
      return {
        text: data.content?.[0]?.text || '',
        provider: 'anthropic',
      };
    }

    if (provider === 'openai') {
      const apiKey = key || '';
      const response = await fetch('https://api.openai.com/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          model,
          max_tokens: maxTokens,
          messages: [
            { role: 'system', content: systemPrompt },
            { role: 'user', content: userMessage },
          ],
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`[openai] ${response.status}: ${errorText}`);
      }

      const data = await response.json();
      return {
        text: data.choices?.[0]?.message?.content || '',
        provider: 'openai',
      };
    }

    if (provider === 'gemini') {
      const apiKey = key || '';
      const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${apiKey}`;
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          system_instruction: { parts: [{ text: systemPrompt }] },
          contents: [{ parts: [{ text: userMessage }] }],
          generationConfig: { maxOutputTokens: maxTokens },
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`[gemini] ${response.status}: ${errorText}`);
      }

      const data = await response.json();
      return {
        text: data.candidates?.[0]?.content?.parts?.[0]?.text || '',
        provider: 'gemini',
      };
    }

    if (provider === 'ollama') {
      const host = ollama_host || 'http://localhost:11434';
      const url = host.endsWith('/') ? `${host}api/chat` : `${host}/api/chat`;
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          model,
          stream: false,
          messages: [
            { role: 'system', content: systemPrompt },
            { role: 'user', content: userMessage },
          ],
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`[ollama] ${response.status}: ${errorText}`);
      }

      const data = await response.json();
      return {
        text: data.message?.content || '',
        provider: 'ollama',
      };
    }

    throw new Error(`Unsupported provider: ${provider}`);
  } catch (err: any) {
    if (err.message && err.message.startsWith(`[${provider}]`)) {
      throw err;
    }
    throw new Error(`[${provider}] Error: ${err.message || err}`);
  }
}
