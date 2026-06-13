export type LLMProvider = 'anthropic' | 'openai' | 'gemini' | 'ollama';

export interface LLMConfig {
  provider: LLMProvider;
  model: string;
}

export interface LLMResult {
  text: string;
  provider: LLMProvider;
}

export function detectProvider(): LLMConfig | null {
  if (process.env.ANTHROPIC_API_KEY) {
    return { provider: 'anthropic', model: 'claude-haiku-3-5' };
  }
  if (process.env.OPENAI_API_KEY) {
    return { provider: 'openai', model: 'gpt-4o-mini' };
  }
  if (process.env.GEMINI_API_KEY) {
    return { provider: 'gemini', model: 'gemini-1.5-flash' };
  }
  if (process.env.OLLAMA_HOST) {
    return { provider: 'ollama', model: process.env.OLLAMA_MODEL || 'llama3' };
  }
  return null;
}

export function getLLMStatus(): { available: boolean; provider: LLMProvider | null; model: string | null } {
  const config = detectProvider();
  if (config) {
    return {
      available: true,
      provider: config.provider,
      model: config.model,
    };
  }
  return {
    available: false,
    provider: null,
    model: null,
  };
}

export async function llmComplete(system: string, user: string, maxTokens: number): Promise<LLMResult> {
  const config = detectProvider();
  if (!config) {
    throw new Error('No LLM provider configured');
  }

  const { provider, model } = config;

  try {
    if (provider === 'anthropic') {
      const apiKey = process.env.ANTHROPIC_API_KEY || '';
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
          system,
          messages: [{ role: 'user', content: user }],
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
      const apiKey = process.env.OPENAI_API_KEY || '';
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
            { role: 'system', content: system },
            { role: 'user', content: user },
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
      const apiKey = process.env.GEMINI_API_KEY || '';
      const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${apiKey}`;
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          system_instruction: { parts: [{ text: system }] },
          contents: [{ parts: [{ text: user }] }],
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
      const host = process.env.OLLAMA_HOST || 'http://localhost:11434';
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
            { role: 'system', content: system },
            { role: 'user', content: user },
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
